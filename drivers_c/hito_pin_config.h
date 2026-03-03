/** Author: Mike Genosse, mike@genosse.org */
#ifndef __hito_pin_config_included__
#define __hito_pin_config_included__

#include "stdbool.h"

#ifdef __ZEPHYR__
#include <device.h>
#endif

//extern bool legacy_bootloader;
// Config pins
void hito_pin_config();

// Set pins initial state
void hito_pin_initial_state();

// Returns true if dev board, detected by pin P0.03 connected to VDD
bool hito_dev_board();

typedef enum {
  HITO_LED_RED   = 0,
  HITO_LED_GREEN = 1,
  HITO_LED_BLUE  = 2,
  HITO_LED_LCD   = 3,
  HITO_LED_DEBUG = 4
} hito_led_t;

void hito_led_on(hito_led_t led);

void hito_led_off();
void hito_led_off_one(hito_led_t led);

// Debug led on
void hito_debug_led_on();

// Debug led off
void hito_debug_led_off();

// Blink debug led n times
void hito_debug_led_blink(char n);

// Firmware and MCU flash config
#define HITO_MCU_FLASH_PAGE_SIZE  4096
#define HITO_FIRMWARE_START_ADDR  0x40000
#define HITO_FIRMWARE_END_ADDR    0xfffff

// LCD - Display pin config
#define HITO_ILI9342_SCK_PIN   DT_PROP(DT_ALIAS(hito_lcd), sck_pin)
#define HITO_ILI9342_MOSI_PIN  DT_PROP(DT_ALIAS(hito_lcd), mosi_pin)
#define HITO_ILI9342_DC_PIN    DT_PROP(DT_PATH(zephyr_user), lcd_dcx_pin)
#define HITO_ILI9342_RESET_PIN DT_PROP(DT_PATH(zephyr_user), lcd_reset_pin)
#define HITO_ILI9342_POWER_PIN DT_PROP(DT_PATH(zephyr_user), lcd_power_pin)
#define HITO_ILI9342_CS_PIN    DT_PROP(DT_PATH(zephyr_user), lcd_cs_pin)
#define HITO_ILI9342_LED_PIN   DT_PROP(DT_PATH(zephyr_user), lcd_led_pin)

// CTP - Capacitive touch panel pin config
#define HITO_FT6306_POWER_PIN DT_PROP(DT_PATH(zephyr_user), ctp_power_pin)
#define HITO_FT6306_SDA_PIN   DT_PROP(DT_ALIAS(hito_ctp), sda_pin)
#define HITO_FT6306_SCL_PIN   DT_PROP(DT_ALIAS(hito_ctp), scl_pin)
#define HITO_FT6306_INT_PIN   DT_PROP(DT_PATH(zephyr_user), ctp_int_pin)
#define HITO_FT6306_RESET_PIN DT_PROP(DT_PATH(zephyr_user), ctp_reset_pin)
#define HITO_FT6306_ADDRESS   DT_PROP(DT_ALIAS(hito_ctp), reg)

// NAND FLASH pin config
#define HITO_NAND_CS_PIN      DT_PROP(DT_PATH(zephyr_user), nand_cs_pin) 
#define HITO_NAND_MISO_PIN    DT_PROP(DT_PATH(zephyr_user), nand_miso_pin)
#define HITO_NAND_HOLD_PIN    DT_PROP(DT_PATH(zephyr_user), nand_hold_pin)
#define HITO_NAND_WP_PIN      DT_PROP(DT_PATH(zephyr_user), nand_hold_pin)
#define HITO_NAND_SCK_PIN     DT_PROP(DT_ALIAS(hito_lcd), sck_pin)
#define HITO_NAND_MOSI_PIN    DT_PROP(DT_ALIAS(hito_lcd), mosi_pin)

// LED pin config
#define HITO_LED_RED_PIN      DT_PROP(DT_PATH(zephyr_user), led_red_pin)     
#define HITO_LED_GREEN_PIN    DT_PROP(DT_PATH(zephyr_user), led_green_pin)
#define HITO_LED_BLUE_PIN     DT_PROP(DT_PATH(zephyr_user), led_blue_pin)     
#define HITO_LED_DEBUG_PIN    DT_PROP(DT_PATH(zephyr_user), led_debug_pin)     

// Charging notification
#define HITO_CHARGING_INPUT_PIN  DT_PROP(DT_PATH(zephyr_user), charge_pin)

// Debug - Dev board detect pin, has to be connected to VDD on Dev Board
#define HITO_DEVBOARD_DETECT_PIN DT_PROP(DT_PATH(zephyr_user), devboard_pin)

// Power button
#define HITO_BUTTON_PIN          DT_PROP(DT_PATH(zephyr_user), button_pin)

#endif//__hito_pin_config_included__
