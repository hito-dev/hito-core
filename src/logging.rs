//! Minimal embedded logging (no_std, no alloc, no threads).
//!
//! Levels:
//! - `error!`        : something failed
//! - `warn!`         : odd but recoverable
//! - `info!`         : expected high-level lifecycle events
//! - `ok!`           : messages to indicate success, marked with green
//! - `debug!` or `d!`: developer-focused diagnostics
//! - `trace!` or `t!`: very noisy, step-by-step flow
//!
//! Compile-time configuraton (feature flags):
//! - `log-error`
//! - `log-warn`
//! - `log-info` (turns on info and ok)
//! - `log-debug`
//! - `log-trace`
//!
//! If none are enabled, defaults to `info`.
//!
//!
//! Goals:
//! - `debug!` / `trace!` compile to **nothing** when disabled (zero code, args not evaluated).
//! - `error!` / `warn!` are always compiled (high-signal).
//! - `info!` is compiled by default, but can be compiled out via a feature.
//! - Runtime: logs can be routed to a sink (UART/USB/RTT) or dropped.
//!
//!
//! Runtime configuration
//! - Call `logging::set_backend(Some(my_backend))` once you have a transport.
//! - Call `logging::set_enabled(false)` to drop everything quickly.

#![allow(dead_code)]
#![allow(unexpected_cfgs)]

use core::fmt;
use core::fmt::Write as _;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Level {
    Error,
    Warn,
    Info,
    Ok,
    Debug,
    Trace,
    BackTrace,
    BootBanner,
}

pub type LogBackend   = fn(&[u8]);   /// writes bytes to output;
pub type TimeProvider = fn() -> u64; /// returns timestamp in ms;

pub static mut BACKEND : Option<LogBackend> = None;
static mut TIME    : Option<TimeProvider> = None;
static mut ENABLED : bool = true;

#[inline(always)]
pub fn set_enabled(enabled: bool) {
    unsafe { ENABLED = enabled; }
}

#[inline(always)]
pub fn set_backend(backend: Option<LogBackend>) {
    unsafe { BACKEND = backend; }
}

#[inline(always)]
pub fn set_time_provider(time_provider: Option<TimeProvider>) {
    unsafe { TIME = time_provider; }
}

#[inline(always)]
pub fn write(level: Level, args: fmt::Arguments) {
    write_ext(level, None, None, args);
}

pub struct HexDump<'a>(pub &'a [u8]);
impl<'a> fmt::Display for HexDump<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, chunk) in self.0.chunks(16).enumerate() {
            // Offset
            write!(f, "{:04x}: ", i * 16)?;

            // Hex bytes in groups of 4
            for (j, byte) in chunk.iter().enumerate() {
                write!(f, "{:02x}", byte)?;
                // Add space after every 4 bytes (but not at the end of the line)
                if (j + 1) % 4 == 0 && j + 1 < 16 {
                    write!(f, " ")?;
                }
            }

            // Padding to align ASCII column
            let bytes_shown = chunk.len();
            let hex_chars = bytes_shown * 2;
            let spaces_shown = if bytes_shown > 4 {
                (bytes_shown - 1) / 4
            } else {
                0
            };
            let total_shown = hex_chars + spaces_shown;
            let padding_needed = 35 - total_shown;

            for _ in 0..padding_needed {
                write!(f, " ")?;
            }

            // ASCII representation
            write!(f, " |")?;
            for &byte in chunk {
                if byte.is_ascii_graphic() || byte == b' ' {
                    write!(f, "{}", byte as char)?;
                } else {
                    write!(f, ".")?;
                }
            }

            // Pad ASCII column too
            for _ in 0..(16 - bytes_shown) {
                write!(f, " ")?;
            }

            writeln!(f, "|")?;
        }

        // Total size at bottom
        let len = self.0.len();
        if len == 1 {
            write!(f, "      ({} byte)", len)?;
        } else {
            write!(f, "      ({} bytes)", len)?;
        }

        Ok(())
    }
}


#[cfg(feature = "simulator")] 
use std::sync::OnceLock;
#[cfg(feature = "simulator")] 
static INITIALIZED: OnceLock<()> = OnceLock::new();

#[inline(always)]
pub fn write_ext(level: Level, file: Option<&'static str>, line: Option<u32>, args: fmt::Arguments) {
    #[cfg(feature = "simulator")]
    INITIALIZED.get_or_init(|| {
        crate::drivers::log_backend::init();
    });

    let (enabled, backend, time) = unsafe { (ENABLED, BACKEND, TIME) };

    if !enabled || backend.is_none() {
        return;
    }

    let mut buf = [0u8; 1024];
    let mut w = BufWriter::new(&mut buf);

    // Prefix
    let _ = w.write_str(match level {
        Level::Error      => "\x1b[31m",    // red
        Level::Warn       => "\x1b[33m",    // yellow
        Level::Info       => "\x1b[0m",     // default
        Level::Ok         => "\x1b[32m",    // green
        Level::Debug      => "\x1b[90m",    // dark gray
        Level::Trace      => "\x1b[36m",    // cyan
        Level::BackTrace  => "\x1b[1;36m",  // bold cyan
        Level::BootBanner => "\x1b[1;37m",  // bold white
    });
    if let Some(time) = time {
        let timestamp_ms = time();
        let seconds = timestamp_ms / 1000;
        let millis  = timestamp_ms % 1000;
        let _ = fmt::write(&mut w, format_args!("[{:>3}.{:03}] ", seconds, millis));
    } else {
        let _ = w.write_str("[     ] ");
    }
    let _ = w.write_str(match level {
        Level::Error => "Error: ",
        Level::Warn  => "Warning: ",
        Level::Info  => "",
        Level::Ok    => "",
        Level::Debug => "",
        Level::Trace => "",
        Level::BackTrace => "",
        Level::BootBanner => "",
    });

    let _ = fmt::write(&mut w, args);

    if let (Some(f), Some(l)) = (file, line) {
        let _ = fmt::write(&mut w, format_args!("\x1b[2m at {}:{}", f, l));
    }

    let _ = w.write_str("\x1b[0m\r\n");
    let out = w.as_bytes();

    if let Some(backend) = backend {
        backend(out);
    }
}

// ---- fixed buffer writer ----

pub struct BufWriter<'a> {
    buf: &'a mut [u8],
    pos: usize,
}

impl<'a> BufWriter<'a> {
    #[inline(always)]
    pub fn new(buf: &'a mut [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    #[inline(always)]
    pub fn as_bytes(&self) -> &[u8] {
        &self.buf[..self.pos]
    }
}

impl fmt::Write for BufWriter<'_> {
    #[inline(always)]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let bytes = s.as_bytes();
        let remaining = self.buf.len().saturating_sub(self.pos);
        let n = core::cmp::min(remaining, bytes.len());
        self.buf[self.pos..self.pos + n].copy_from_slice(&bytes[..n]);
        self.pos += n;
        Ok(())
    }
}

#[cfg(feature = "simulator")]
pub fn print_backtrace(w: &mut impl fmt::Write) {

    extern crate std;
    use std::format;
    use std::string::ToString;

    use backtrace::Backtrace;
    let bt = Backtrace::new();
    let frames = bt.frames();

    // Skip the first few frames that are just logging internals
    let interesting = frames
        .iter()
        .flat_map(|f| f.symbols())
        .filter(|s| {
            s.name().map_or(false, |n| {
                let name = format!("{n}");  // <- fix here
                !name.contains("logging")
                    && !name.contains("backtrace")
                    && !name.contains("__rust")
                    && !name.contains("std::rt")
            })
        })
        //.skip(1) // skip the caller of backtrace!() itself
        .take_while(|s| {
            s.name().map_or(true, |n| {
                !std::format!("{n}").starts_with("core::ops")
            })
        });
        //.collect();
        //.collect();


    let _ = w.write_str("\x1b[90m");
    for (i, symbol) in interesting.enumerate() {
        // let name = symbol
        //     .name()
        //     .map_or("<unknown>".into(), |n| n.to_string());
        let name = symbol.name().map_or("<unknown>".to_string(), |n| {
            let name = std::format!("{n}");
            // strip ::h<16 hex chars> mangling suffix
            if let Some(pos) = name.rfind("::h") {
                let suffix = &name[pos + 3..];
                if suffix.len() == 16 && suffix.chars().all(|c| c.is_ascii_hexdigit()) {
                    return name[..pos].to_string();
                }
            }
            name
        });
        let file = strip_path_prefix(symbol.filename().and_then(|p| p.to_str()).unwrap_or("?"));
        let line = symbol.lineno().unwrap_or(0);
        //let _ = fmt::write(w, format_args!("      #{i} .. {name}\tat {file}:{line}\n"));
        let _ = std::fmt::write(w, std::format_args!(
            "      \x1b[90m#{i} \x1b[90m.. \x1b[1;36m{name}\x1b[0m\t\x1b[90mat {file}:{line}\x1b[0m\n"
        ));

    }
    let _ = w.write_str("\x1b[0m");
}

#[cfg(not(feature = "simulator"))]
#[inline(always)]
pub fn print_backtrace(_w: &mut impl fmt::Write) {
    // no-op on real hardware
}

// ---------------- Path helpers ----------------

/// Strip common path prefixes to get relative paths
#[inline]
//pub fn strip_path_prefix(path: &'static str) -> &'static str {
pub fn strip_path_prefix(path: &str) -> &str {
    // Find first occurrence of "/src/", "/crates/", or "/apps/"
    if let Some(pos) = path.find("crates/") {
        return &path[pos..];
    }
    if let Some(pos) = path.find("apps/") {
        return &path[pos..];
    }
    if let Some(pos) = path.find("src/") {
        return &path[pos..];
    }
    path
}

// ---------------- Macros ----------------
#[macro_export]
macro_rules! hexdump {
    ($data:expr) => {{
        $crate::logging::HexDump($data)
    }};
}

/// Error (compiled when level >= error; no file:line by default)
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {{
        #[cfg(any(feature = "log-error", feature = "log-warn", feature = "log-info", feature = "log-debug", feature = "log-trace"))]
        $crate::logging::write(
            $crate::logging::Level::Error, 
            core::format_args!($($arg)*)
        );
    }};
}

/// Warn (compiled when level >= warn; no file:line by default)
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {{
        #[cfg(any(feature = "log-warn", feature = "log-info", feature = "log-debug", feature = "log-trace"))]
        $crate::logging::write(
            $crate::logging::Level::Warn, 
            core::format_args!($($arg)*)
        );
    }};
}

/// Info (compiled when level >= info; no file:line by default)
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {{
        #[cfg(any(feature = "log-info", feature = "log-debug", feature = "log-trace"))]
        $crate::logging::write(
            $crate::logging::Level::Info, 
            core::format_args!($($arg)*)
        );
    }};
}
#[macro_export]
macro_rules! ok {
    ($($arg:tt)*) => {{
        #[cfg(any(feature = "log-info", feature = "log-debug", feature = "log-trace"))]
        $crate::logging::write(
            $crate::logging::Level::Ok, 
            core::format_args!($($arg)*)
        );
    }};
}

#[macro_export]
macro_rules! log_boot_banner {
    () => {
        #[allow(unexpected_cfgs)]
        #[cfg(any(feature = "log-info", feature = "log-debug", feature = "log-trace"))]
        $crate::logging::write(
            $crate::logging::Level::BootBanner, 
            core::format_args!("{} v{} {}/{}{}",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION"),
                match () {
                    _ if cfg!(feature = "zephyr")    => "zephyr",
                    _                                => "simulator",
                },
                match () {
                    _ if cfg!(feature = "log-trace") => "log-trace",
                    _ if cfg!(feature = "log-debug") => "log-debug",
                    _ if cfg!(feature = "log-error") => "log-debug",
                    _ if cfg!(feature = "log-warn")  => "log-warn",
                    _ if cfg!(feature = "log-info")  => "log-info",
                    _                                => "no-log",
                },
                match () {
                    _ if cfg!(feature = "gui")     => "/gui",
                    _ if cfg!(feature = "console") => "/console",
                    _                              => "",
                },
            ),
        );
    }
}

/// Debug (compiled when level >= debug; includes file:line)
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {{
        #[cfg(any(feature = "log-debug", feature = "log-trace"))]
        $crate::logging::write_ext(
            $crate::logging::Level::Debug, 
            Some($crate::logging::strip_path_prefix(file!())), 
            Some(line!()),
            core::format_args!($($arg)*)
        );
    }};
}

/// Trace (compiled when level >= trace; includes file:line)
#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {{
        #[cfg(any(feature = "log-trace"))]
        $crate::logging::write_ext(
            $crate::logging::Level::Trace, 
            Some($crate::logging::strip_path_prefix(file!())), 
            Some(line!()),
            core::format_args!($($arg)*)
        );
    }};
}

#[macro_export]
macro_rules! backtrace {
    ($($arg:tt)*) => {{
        #[cfg(any(feature = "log-trace"))]
        {
            #[cfg(feature = "simulator")]
            {
                $crate::logging::write(
                    $crate::logging::Level::BackTrace,
                    //Some($crate::logging::strip_path_prefix(file!())),
                    //Some(line!()),
                    core::format_args!($($arg)*),
                );
                // print backtrace to a temporary buffer and flush via backend
                let mut buf = [0u8; 4096];
                let mut w = $crate::logging::BufWriter::new(&mut buf);
                $crate::logging::print_backtrace(&mut w);
                let out = w.as_bytes();
                unsafe {
                    if let Some(backend) = $crate::logging::BACKEND {
                        backend(out);
                    }
                }
            }
            #[cfg(not(feature = "simulator"))]
            {
                $crate::logging::write_ext(
                    $crate::logging::Level::BackTrace,
                    Some($crate::logging::strip_path_prefix(file!())),
                    Some(line!()),
                    core::format_args!($($arg)*),
                );
            }
        }
    }};
}

// Shortened aliases (optional)
#[macro_export] macro_rules! err { ($($arg:tt)*) => { $crate::error!($($arg)*) }; }
#[macro_export] macro_rules! d   { ($($arg:tt)*) => { $crate::debug!($($arg)*) }; }
#[macro_export] macro_rules! t   { ($($arg:tt)*) => { $crate::trace!($($arg)*) }; }
#[macro_export] macro_rules! bt  { ($($arg:tt)*) => { $crate::backtrace!($($arg)*) }; }
