use crate::driver;

driver! {
    pub trait TimeDriver => Time {
        fn now_ms() -> u64;
        fn sleep_ms(ms: u32);
        fn sleep_us(ms: u32);
    }
}

#[cfg(feature = "simulator")]
pub use crate::drivers::simulator::time::TimeSimulator as Time;

#[cfg(feature = "zephyr")]
pub use crate::drivers::zephyr::time::TimeZephyr as Time;

