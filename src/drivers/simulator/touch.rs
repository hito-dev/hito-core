// touch.rs
use crate::simulator_window::*;

pub struct TouchMinifb;

impl crate::drivers::touch::TouchDriver for TouchMinifb {

    fn init() -> bool  { 
        simulator_window_init();
        true
    }

    //fn has_touch() -> bool { 
        //simulator_window_has_touch()
    //}

    fn xy() -> Option<(u16, u16)> {
        if !simulator_window_has_touch() {
            return None;
        }
        let (x, y) = simulator_window_touch_position();
        Some((x, y))
    }

}


