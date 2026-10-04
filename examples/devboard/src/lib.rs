#![no_std]

use hito_core::drivers::{Display, Time, Touch};

hito_core::hito_main!(run);

fn run() {
    Display::init();
    Touch::init();
    Display::fill_screen(0x0000);
    Display::fill_rect(20, 20, 280, 30, 0x07e0);
    hito_core::info!("Hito Core devboard ready; touch the display to draw");
    loop {
        if let Some((x, y)) = Touch::xy() {
            if x < Display::WIDTH - 4 && y < Display::HEIGHT - 4 {
                Display::fill_rect(x, y, 4, 4, 0xffff);
            }
        }
        Time::sleep_ms(10);
    }
}
