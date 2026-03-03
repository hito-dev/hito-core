use std::{
    fs,
    io::{BufRead, BufReader, Write},
    os::unix::net::{UnixListener, UnixStream},
    path::Path,
    sync::{Condvar, Mutex, OnceLock},
    thread,
    vec::Vec,
    string::String,
};

use crate::drivers::serial::SerialDriver;

const SOCK_PATH: &str = "/tmp/hito_simulator_transport.sock";
const MAX_LINE: usize = 64 * 1024;

pub struct SerialDesktop;

struct Inner {
    rx: Vec<u8>,
    tx_stream: Option<UnixStream>, // stream we reply on
    started: bool,
}

// Global singleton: (mutex + condvar)
fn state() -> &'static (Mutex<Inner>, Condvar) {
    static STATE: OnceLock<(Mutex<Inner>, Condvar)> = OnceLock::new();
    STATE.get_or_init(|| {
        (
            Mutex::new(Inner {
                rx: Vec::new(),
                tx_stream: None,
                started: false,
            }),
            Condvar::new(),
        )
    })
}

/*
pub trait Serial {
    fn init();
    fn has_data() -> bool;
    fn get_data_len() -> usize;
    fn get_data(out: &mut [u8]) -> usize;
    fn clear_data();
    fn send(data: &[u8]) -> bool;
}
*/

impl SerialDriver for SerialDesktop {
    fn init() -> bool {
        let (lock, _cv) = state();
        let mut g = lock.lock().unwrap();
        if g.started {
            return false;
        }
        g.started = true;
        drop(g);

        if Path::new(SOCK_PATH).exists() {
            let _ = fs::remove_file(SOCK_PATH);
        }

        let listener = UnixListener::bind(SOCK_PATH).expect("bind serial simulator socket");

        thread::spawn(move || {
            for conn in listener.incoming() {
                let stream = match conn {
                    Ok(s) => s,
                    Err(_) => continue,
                };

                // Read exactly one line from the connection.
                // Use a clone for reading so we keep `stream` for writing later.
                let reader_stream = match stream.try_clone() {
                    Ok(s) => s,
                    Err(_) => continue,
                };
                let mut reader = BufReader::new(reader_stream);

                let mut line = String::new();
                let n = match reader.read_line(&mut line) {
                    Ok(n) => n,
                    Err(_) => continue,
                };
                if n == 0 {
                    continue;
                }
                if line.len() > MAX_LINE {
                    continue;
                }
                while line.ends_with('\n') || line.ends_with('\r') {
                    line.pop();
                }
                if line.is_empty() {
                    continue;
                }

                // Store RX + store stream for TX, then WAIT until main replies.
                let (lock, cv) = state();
                let mut g = lock.lock().unwrap();

                // If previous packet not processed yet, we drop this request (device backpressure).
                if !g.rx.is_empty() || g.tx_stream.is_some() {
                    continue;
                }

                g.rx = line.into_bytes();
                g.tx_stream = Some(stream);
                cv.notify_all();

                // Exclusive over time:
                // don't accept the next client until main sent (tx_stream becomes None)
                // and RX cleared (rx becomes empty).
                while !g.rx.is_empty() || g.tx_stream.is_some() {
                    g = cv.wait(g).unwrap();
                }
            }
        });
        true
    }

    fn has_data() -> bool {
        let (lock, _) = state();
        !lock.lock().unwrap().rx.is_empty()
    }

    fn get_data_len() -> usize {
        let (lock, _) = state();
        lock.lock().unwrap().rx.len()
    }

    fn get_data(out: &mut [u8]) -> usize {
        let (lock, _) = state();
        let g = lock.lock().unwrap();
        let n = out.len().min(g.rx.len());
        out[..n].copy_from_slice(&g.rx[..n]);
        n
    }

    fn clear_data() {
        let (lock, cv) = state();
        let mut g = lock.lock().unwrap();
        g.rx.clear();
        cv.notify_all();
    }

    fn has_line() -> bool {
        let (lock, _) = state();
        let g = lock.lock().unwrap();
        if g.rx.is_empty() {
            return false;
        }
        g.rx.last() == Some(&b'\n')
    }

    fn get_line() -> Option<String> {
        let (lock, _) = state();
        let g = lock.lock().unwrap();
        if g.rx.is_empty() {
            return None;
        }
        match String::from_utf8(g.rx.clone()) {
            Ok(s) => Some(s),
            Err(_) => None,
        }
    }

    /// Send reply to the *current* connected client (same socket connection).
    /// Appends '\n'. Closes connection after sending (device-like request/response).
    fn send(data: &[u8]) -> bool {
        let (lock, cv) = state();
        let mut g = lock.lock().unwrap();

        let mut stream = match g.tx_stream.take() {
            Some(s) => s,
            None => return false,
        };

        // Write outside lock? We can, but keep it simple:
        // small writes, low contention. If you prefer, I can refactor.
        let ok = stream.write_all(data).is_ok()
            && stream.write_all(b"\n").is_ok()
            && stream.flush().is_ok();

        // Drop stream = close connection
        drop(stream);

        cv.notify_all();
        ok
    }
}

