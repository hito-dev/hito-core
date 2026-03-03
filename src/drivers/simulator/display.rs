// display.rs

pub struct DisplayMinifb;

//use crate::{drivers::display::Rect};
use crate::simulator_window::*;

impl crate::drivers::display::DisplayDriver for DisplayMinifb {

    fn init() -> bool  { 
        simulator_window_init();
        true
    }

    fn set_brightness(_brightness: u8) {
        // No-op for simulator
    }

    fn draw_line_buffer(x: u16, y: u16, pixels: &[u16]) {
        simulator_window_draw_line(x, y, pixels.len() as u16, pixels);
    }

}


