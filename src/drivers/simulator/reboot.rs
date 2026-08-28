pub struct RebootSimulator;

use std::process::exit;

impl crate::drivers::reboot::RebootDriver for RebootSimulator {
    fn reboot() {
        exit(0);
    }
}
