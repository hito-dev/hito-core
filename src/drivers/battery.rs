use crate::driver;

driver! {
    pub trait BatteryDriver => Battery {
        fn get_battery_level() -> u8;
    }
}

#[cfg(feature = "simulator")]
pub use crate::drivers::simulator::battery::BatterySimulator as Battery;

#[cfg(feature = "zephyr")]
pub use crate::drivers::zephyr::battery::BatteryZephyr as Battery;