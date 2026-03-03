// touch.rs

pub struct TouchZephyr;

extern "C" {
    fn ft6336_ctp_init();
    fn ft6336_ctp_power_on();
    fn ft6336_ctp_has_touch() -> bool;
    fn ft6336_ctp_read_touch() -> bool;
    fn ft6336_ctp_touch_x() -> u16;
    fn ft6336_ctp_touch_y() -> u16;
}

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
                let x = ft6336_ctp_touch_x();
                let y = ft6336_ctp_touch_y();
                Some((x, y))
            } else {
                None
            }
        }
    }
}


