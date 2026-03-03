#ifndef __ili9342_lcd_included__
#define __ili9342_lcd_included__

#include "stdint.h"
#include "stdbool.h"

void ili9342_lcd_spi_init();

//-----------------------------------------------------------------------------
void ili9342_lcd_reset();
void ili9342_lcd_init();

//-----------------------------------------------------------------------------
void ili9342_lcd_set_brightness(uint8_t brightness);

//-----------------------------------------------------------------------------
void ili9342_lcd_led_on();
void ili9342_lcd_led_off();

//-----------------------------------------------------------------------------
void ili9342_lcd_sleep_in();
void ili9342_lcd_sleep_out();
void ili9342_lcd_display_on();
void ili9342_lcd_display_off();
void ili9342_lcd_power_on();
void ili9342_lcd_power_off();

bool ili9342_lcd_draw_qr(const uint8_t * qr_data, uint16_t x, uint16_t y, 
    const uint8_t qr_width, uint16_t image_max_width, bool center);

//-----------------------------------------------------------------------------
void ili9342_lcd_fill_rect(uint16_t x, uint16_t y, uint16_t w, uint16_t h, 
                                 uint16_t color565);

//-----------------------------------------------------------------------------
void ili9342_lcd_draw_screen_corners(bool allCorners);

//-----------------------------------------------------------------------------
void ili9342_lcd_draw_bitmap(uint16_t x, uint16_t y, uint16_t width, uint16_t height,
  const uint8_t * bitmap, uint16_t bitmap_row_pitch);

void ili9342_lcd_draw_line_buffer(uint16_t x, uint16_t y, uint16_t w, const uint8_t * buf);

//-----------------------------------------------------------------------------
//ret_code_t ili9342_lcd_write_command(const char * cmd);

//-----------------------------------------------------------------------------
//ret_code_t ili9342_lcd_set_addr_window(uint16_t x_0, uint16_t y_0, 
                                   //uint16_t x_1, uint16_t y_1);

#endif //__ili9342_lcd_included__

