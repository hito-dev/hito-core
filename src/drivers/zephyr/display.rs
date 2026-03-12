// display.rs

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
        unsafe {
            ili9342_lcd_set_brightness(brightness);
        }
    }

}


