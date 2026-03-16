use crate::{driver};

//use libm::sqrtf;

pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

//pub static mut LINE_BUFFER: [u16; 320] = [0; 320];
//
//pub const WIDTH:  u16 = 320;
//pub const HEIGHT: u16 = 240;

driver! {
    pub trait DisplayDriver => Display {
        fn init() -> bool;   
        fn set_brightness(brightness: u8);
        fn draw_line_buffer(x: u16, y: u16, pixels: &[u16]);
    }
}

fn my_sqrt(x: f32) -> f32 {
    //#[cfg(feature = "zephyr")]
    //return libm::sqrtf(x);

    //#[cfg(feature = "simulator")]
    //return x.sqrt();
    x
}

impl Display {
    pub const WIDTH:  u16 = 320;
    pub const HEIGHT: u16 = 240;

    pub fn fill_screen(color: u16) {
        let buf = [color; Display::WIDTH as usize];
        for i in 0..Display::HEIGHT {
            Self::draw_line_buffer(0, i, &buf);
        }
    }

    pub fn fill_rect(x: u16, y: u16, width: u16, height: u16, color: u16) {
        let buf = [color; Display::WIDTH as usize];
        for i in 0..height {
            Self::draw_line_buffer(x, y + i, &buf[..width as usize]);
        }
    }

    pub fn fill_circle(cx: u16, cy: u16, radius: u16, color: u16) {

        if cx < radius || cx + radius >= Display::WIDTH || cy < radius || cy + radius >= Display::HEIGHT {
            return;
        }

        let buf = [color; Display::WIDTH as usize];

        let r = radius as i32;
        for i in 0..radius * 2 {
            let dy = i as i32 - r;
            let chord = my_sqrt((r * r - dy * dy) as f32) as u16 * 2;
            let x = cx - chord / 2;
            let y = cy - radius + i;
            Self::draw_line_buffer(x, y, &buf[..chord as usize]);
        }
    }
}

#[cfg(feature = "simulator")]
pub use crate::drivers::simulator::display::DisplayMinifb as Display;

#[cfg(feature = "zephyr")]
pub use crate::drivers::zephyr::display::DisplayZephyr as Display;

