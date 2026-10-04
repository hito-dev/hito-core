extern "C" {
    fn hito_power_reboot() -> bool;
}

pub struct RebootZephyr;

impl crate::drivers::reboot::RebootDriver for RebootZephyr {
    fn reboot() {
        unsafe {
            if !hito_power_reboot() {
                // If reboot fails, log and halt
                crate::error!("Device reboot failed");
                loop {}
            }
        }
    }
}
