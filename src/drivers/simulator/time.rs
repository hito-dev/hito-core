use std::sync::OnceLock;
use std::time::Instant;

use crate::drivers::time::TimeDriver;
use crate::simulator_window::{simulator_window_update};

pub struct TimeSimulator;

impl TimeDriver for TimeSimulator {
    fn now_ms() -> u64 {
        boot().elapsed().as_millis() as u64
    }

    fn sleep_ms(ms: u32) {
        simulator_window_update();
        std::thread::sleep(std::time::Duration::from_millis(ms as u64));

        // check for keyboard input while sleeping and exit if 'q' is pressed
        //if simulator_window_is_q_pressed() {
            //std::process::exit(0);  
        //}
    }
}

fn boot() -> &'static Instant {
    static BOOT: OnceLock<Instant> = OnceLock::new();
    BOOT.get_or_init(Instant::now)
}


