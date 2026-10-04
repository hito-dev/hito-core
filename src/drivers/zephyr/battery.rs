// extern "C" {
//     fn hito_battery_level() -> i32;
// }

pub struct BatteryZephyr;

impl crate::drivers::battery::BatteryDriver for BatteryZephyr {
    fn get_battery_level() -> u8 {
        unsafe { hito_battery_level() as u8 }
    }
}