use crate::logging::LogBackend;
use crate::logging::TimeProvider;

use crate::drivers::Time;

#[cfg(feature = "simulator")]
use std::{
    print,
    sync::OnceLock,
    time::SystemTime,
};

pub fn default_backend() -> LogBackend {

    fn backend(bytes: &[u8]) {
        #[cfg(feature = "simulator")]
        if let Ok(s) = core::str::from_utf8(bytes) {
            print!("{}", s);
        }

        #[cfg(feature = "zephyr")]
        {
            extern "C" {
                //fn printk(fmt: *const u8, ...) -> i32;
                fn hito_platform_log(msg: *const u8, msg_len: u16);
            }
            unsafe {
                //printk(b"%.*s\0".as_ptr(), bytes.len() as i32, bytes.as_ptr());
                hito_platform_log(bytes.as_ptr(), bytes.len() as u16);
            }
        }
    }
    backend
}

#[cfg(feature = "simulator")]
static START: OnceLock<SystemTime> = OnceLock::new();

pub fn default_time_provider() -> TimeProvider {

    fn now() -> u64 {
        Time::now_ms()
    }
    now
}

pub fn init() {
    crate::logging::set_backend(Some(default_backend()));
    crate::logging::set_time_provider(Some(default_time_provider()));
    //ok!("{} init", module_path!());
    //log_boot_banner!();
}

