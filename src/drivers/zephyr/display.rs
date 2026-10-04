// display.rs

const BRIGHTNESS_CHECKSUM_ADDR: usize =
    0x20000000 + 0x6f800 + 2046;

const BRIGHTNESS_ADDR: usize =
    0x20000000 + 0x6f800 + 2047;

const BRIGHTNESS_CHECKSUM_MAGIC: u8 = 10;
const DEFAULT_BRIGHTNESS: u8 = 95;

fn brightness_save(new_brightness: u8) {
    unsafe {
        core::ptr::write_volatile(
            BRIGHTNESS_ADDR as *mut u8,
            new_brightness,
        );

        core::ptr::write_volatile(
            BRIGHTNESS_CHECKSUM_ADDR as *mut u8,
            new_brightness.wrapping_add(BRIGHTNESS_CHECKSUM_MAGIC),
        );
        ili9342_lcd_set_brightness(new_brightness);
    }
}

fn brightness_load() -> Option<u8> {
    unsafe {
        let brightness =
            core::ptr::read_volatile(BRIGHTNESS_ADDR as *const u8);

        let checksum =
            core::ptr::read_volatile(BRIGHTNESS_CHECKSUM_ADDR as *const u8);

        if checksum == brightness.wrapping_add(BRIGHTNESS_CHECKSUM_MAGIC) {
            Some(brightness)
        } else {
            None
        }
    }
}

pub struct DisplayZephyr;

//use crate::drivers::display::rect;

extern "C" {
    fn ili9342_lcd_init();
    fn ili9342_lcd_set_brightness(brightness: u8);
    //fn ili9342_lcd_fill_rect(x: u16, y: u16, width: u16, height: u16, color: u16);
    //fn ili9342_lcd_draw_screen_corners(inverted: bool);
    fn ili9342_lcd_draw_line_buffer(x: u16, y: u16, w: u16, buf: *const u16);

    fn hito_pin_config();
}

impl crate::drivers::display::DisplayDriver for DisplayZephyr {

    fn init() -> bool  { 

        unsafe {
            //hito_pin_config();
            ili9342_lcd_init();

            match brightness_load() {
                Some(brightness) => {
                    ili9342_lcd_set_brightness(brightness);
                }
                None => {
                    ili9342_lcd_set_brightness(DEFAULT_BRIGHTNESS);
                }
            }

            //ili9342_lcd_fill_rect(0, 0, 320, 240, 0xffff);

            //ili9342_lcd_fill_rect(50, 50, 50, 50, 0x0488);
            //ili9342_lcd_draw_screen_corners(false);

            //ili9342_lcd_set_brightness(100);
        }
        true
    }

    #[inline(always)]
    fn draw_line_buffer(x: u16, y: u16, pixels: &[u16]) {
        unsafe {
            ili9342_lcd_draw_line_buffer(x, y, pixels.len() as u16, pixels.as_ptr());
            //ili9342_lcd_fill_rect(x, y, 40, 40, 0x0488);
        }
    }

    fn set_brightness(brightness: u8) {
        brightness_save(brightness);
    }

}


