
use crate::drivers::time::TimeDriver;

pub struct TimeZephyr;

extern "C" {
    fn hito_platform_time_sleep_ms(ms: u32);
    fn hito_platform_time_uptime_ms() -> i64;
}

impl TimeDriver for TimeZephyr {
    fn now_ms() -> u64 { unsafe {
        hito_platform_time_uptime_ms() as u64
    }}

    fn sleep_ms(ms: u32) { unsafe {
        hito_platform_time_sleep_ms(ms);
    }}
}


