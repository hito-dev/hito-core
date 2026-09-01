use rand::Rng;

pub struct BatterySimulator;

impl crate::drivers::battery::BatteryDriver for BatterySimulator {
    fn get_battery_level() -> u8 {
        let level = rand::rng().random_range(0..=100);
        info!("Battery level: {}%", level);
        level
    }
}