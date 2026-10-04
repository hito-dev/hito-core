// touch.rs
use crate::drivers::time::Time;

pub struct TouchZephyr;

extern "C" {
    fn ft6336_ctp_init();
    fn ft6336_ctp_power_on();
    fn ft6336_ctp_has_touch() -> bool;
    fn ft6336_ctp_read_touch() -> bool;
    fn ft6336_ctp_touch_x() -> u16;
    fn ft6336_ctp_touch_y() -> u16;
}

static mut LAST_TOUCH_TIME: u64 = 0;

impl crate::drivers::touch::TouchDriver for TouchZephyr {

    fn init() -> bool  { 
        unsafe { 
            ft6336_ctp_init();
            ft6336_ctp_power_on();
        }
        true
    }

    fn xy() -> Option<(u16, u16)> {
        unsafe {
            if !ft6336_ctp_has_touch() {
                return None;
            }
            if ft6336_ctp_read_touch() {
                LAST_TOUCH_TIME = Time::now_ms();
            }
            // We need to use time to prevent flickering on real device
            if Time::now_ms() - LAST_TOUCH_TIME < 100 {
                let x = ft6336_ctp_touch_x();
                let y = ft6336_ctp_touch_y();
                Some((x, y))
            } else {
                None
            }
        }
    }
}


