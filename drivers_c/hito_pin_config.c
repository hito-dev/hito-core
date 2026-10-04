/** Author: Mike Genosse, mike@genosse.org */
#include "hito_pin_config.h"

#include <device.h>
#include <hal/nrf_gpio.h>
#include <sys/crc.h>

#include <hal/nrf_vmc.h>

#include "crypt0_log.h"
LOG_MODULE_REGISTER(hito_pin_config, LOG_LEVEL_DBG);

// legacy bootloader has different configuration of pins 
// - blue & green pins are messed up
static bool legacy_bootloader = false;

void check_legacy_bootloader() {
  uint16_t crc = crc16_ccitt(0xffff, 0x0, 4096);
  if (crc == 0x4143) {
    legacy_bootloader = true;
  }
}

void ram_retention_set() 
{
  // section start CONFIG_SRAM_BASE_ADDRESS + 0x6f800
  // section size 2048
  nrf_vmc_ram_block_retention_set(NRF_VMC, 6, 0x8000ffff);
}

void set_regout_3v3() 
{

  NRF_NVMC->CONFIG = NVMC_CONFIG_WEN_Wen << NVMC_CONFIG_WEN_Pos;
  while (NRF_NVMC->READY == NVMC_READY_READY_Busy){}
#ifdef UICR_VREGHVOUT_VREGHVOUT_3V3
  NRF_UICR->VREGHVOUT = UICR_VREGHVOUT_VREGHVOUT_3V3;
#else
  NRF_UICR->REGOUT0 = UICR_REGOUT0_VOUT_3V3;
#endif
  NRF_NVMC->CONFIG = NVMC_CONFIG_WEN_Ren << NVMC_CONFIG_WEN_Pos;
  while (NRF_NVMC->READY == NVMC_READY_READY_Busy){}
}

//-----------------------------------------------------------------------------
void hito_pin_config()
{
  LOG_DBG("> hito_pin_config");
  set_regout_3v3();

  ram_retention_set();

  // avoid glitching
  hito_pin_initial_state();

  check_legacy_bootloader();

  // dev board
  //nrf_gpio_cfg_input(HITO_DEVBOARD_DETECT_PIN, NRF_GPIO_PIN_PULLDOWN);

  // lcd
  nrf_gpio_cfg_output(HITO_ILI9342_SCK_PIN);
  nrf_gpio_cfg_output(HITO_ILI9342_MOSI_PIN);
  nrf_gpio_cfg_output(HITO_ILI9342_DC_PIN);
  nrf_gpio_cfg_output(HITO_ILI9342_RESET_PIN);
  nrf_gpio_cfg_output(HITO_ILI9342_CS_PIN);
  nrf_gpio_cfg_output(HITO_ILI9342_LED_PIN);
  nrf_gpio_cfg(
      HITO_ILI9342_POWER_PIN,
      NRF_GPIO_PIN_DIR_OUTPUT,
      NRF_GPIO_PIN_INPUT_DISCONNECT,
      NRF_GPIO_PIN_PULLDOWN,
      NRF_GPIO_PIN_S0H1,
      NRF_GPIO_PIN_NOSENSE);

  // ctp
  nrf_gpio_cfg_output(HITO_FT6306_SDA_PIN);
  nrf_gpio_cfg_output(HITO_FT6306_SCL_PIN);
  //nrf_gpio_cfg_input(HITO_FT6306_INTERRUPT_PIN, NRF_GPIO_PIN_PULLDOWN);
  nrf_gpio_cfg_output(HITO_FT6306_INT_PIN);
  nrf_gpio_cfg_output(HITO_FT6306_RESET_PIN);
  nrf_gpio_cfg(
      HITO_FT6306_POWER_PIN,
      NRF_GPIO_PIN_DIR_OUTPUT,
      NRF_GPIO_PIN_INPUT_DISCONNECT,
      NRF_GPIO_PIN_PULLDOWN,
      NRF_GPIO_PIN_S0H1,
      NRF_GPIO_PIN_NOSENSE);

  // led
  nrf_gpio_cfg_output(HITO_LED_RED_PIN);
  nrf_gpio_cfg_output(HITO_LED_GREEN_PIN);
  nrf_gpio_cfg_output(HITO_LED_BLUE_PIN);
  //if (hito_dev_board()) {
    //nrf_gpio_cfg_output(HITO_LED_DEBUG_PIN);
  //} else {
    // no output on real device because led is shared with charging input
  //}

  // memory
  nrf_gpio_cfg_output(HITO_NAND_CS_PIN);
  //nrf_gpio_cfg_input(HITO_NAND_MISO_PIN, NRF_GPIO_PIN_PULLDOWN);
  nrf_gpio_cfg_output(HITO_NAND_MISO_PIN);
  nrf_gpio_cfg_output(HITO_NAND_HOLD_PIN);
  nrf_gpio_cfg_output(HITO_NAND_WP_PIN);
  nrf_gpio_cfg_output(HITO_NAND_MOSI_PIN);
  nrf_gpio_cfg_output(HITO_NAND_SCK_PIN);

  // charging
  nrf_gpio_cfg_input(HITO_CHARGING_INPUT_PIN, NRF_GPIO_PIN_PULLUP);
  //nrf_gpio_cfg_sense_input(HITO_CHARGING_INPUT_PIN, NRF_GPIO_PIN_PULLUP, NRF_GPIO_PIN_SENSE_LOW);
	nrf_gpio_cfg_sense_set(HITO_CHARGING_INPUT_PIN, NRF_GPIO_PIN_SENSE_LOW);

  // button
  nrf_gpio_cfg_input(HITO_BUTTON_PIN, NRF_GPIO_PIN_PULLUP);
	nrf_gpio_cfg_sense_set(HITO_BUTTON_PIN, NRF_GPIO_PIN_SENSE_LOW);

  // set pin initial state
  hito_pin_initial_state();
  LOG_DBG("< hito_pin_config");
}

//-----------------------------------------------------------------------------
void hito_pin_initial_state()
{
  k_msleep(10);
  //int a = DT_PROP(DT_ALIAS(hito_ctp), reg);

  // lcd
  nrf_gpio_pin_clear(HITO_ILI9342_SCK_PIN);
  nrf_gpio_pin_clear(HITO_ILI9342_MOSI_PIN);
  nrf_gpio_pin_clear(HITO_ILI9342_DC_PIN);
  nrf_gpio_pin_clear(HITO_ILI9342_RESET_PIN);
  nrf_gpio_pin_clear(HITO_ILI9342_CS_PIN);
  nrf_gpio_pin_clear(HITO_ILI9342_LED_PIN);
  nrf_gpio_pin_clear(HITO_ILI9342_POWER_PIN);

  // ctp
  nrf_gpio_pin_clear(HITO_FT6306_SDA_PIN);
  nrf_gpio_pin_clear(HITO_FT6306_SCL_PIN);
  nrf_gpio_pin_clear(HITO_FT6306_RESET_PIN);
  nrf_gpio_pin_clear(HITO_FT6306_POWER_PIN);
  nrf_gpio_pin_clear(HITO_FT6306_INT_PIN);

  // led
  nrf_gpio_pin_set(HITO_LED_RED_PIN);
  nrf_gpio_pin_set(HITO_LED_GREEN_PIN);
  nrf_gpio_pin_set(HITO_LED_BLUE_PIN);
  //if (hito_dev_board()) {
    //nrf_gpio_pin_set(HITO_LED_DEBUG_PIN); // not connected to charging
  //}

  // memory
  nrf_gpio_pin_set(HITO_NAND_CS_PIN);
  nrf_gpio_pin_clear(HITO_NAND_MISO_PIN);
  nrf_gpio_pin_clear(HITO_NAND_MOSI_PIN);
  nrf_gpio_pin_set(HITO_NAND_HOLD_PIN);
  nrf_gpio_pin_set(HITO_NAND_WP_PIN);
  nrf_gpio_pin_clear(HITO_NAND_SCK_PIN);

  //nrf_gpio_pin_set(HITO_NAND_CS_PIN);
  //nrf_gpio_pin_clear(HITO_NAND_MOSI_PIN);
  //nrf_gpio_pin_clear(HITO_NAND_HOLD_PIN);
  //nrf_gpio_pin_set(HITO_NAND_WP_PIN);
  //nrf_gpio_pin_clear(HITO_NAND_SCK_PIN);
  k_msleep(10);
}

//-----------------------------------------------------------------------------
void hito_debug_led_on() {
  return;
  if (hito_dev_board()) {
    nrf_gpio_pin_clear(HITO_LED_DEBUG_PIN);
  } else {
    // on real device debug led is connected to charging input pin
    nrf_gpio_cfg_input(HITO_CHARGING_INPUT_PIN, NRF_GPIO_PIN_PULLDOWN);
    //nrf_gpio_cfg_output(HITO_CHARGING_INPUT_PIN);
    //nrf_gpio_pin_clear(HITO_CHARGING_INPUT_PIN);
  }
}

//-----------------------------------------------------------------------------
void hito_debug_led_off() {
  return;
  if (hito_dev_board()) {
    nrf_gpio_pin_set(HITO_LED_DEBUG_PIN);
  } else {
    // on real device debug led is connected to charging input pin
    nrf_gpio_cfg_input(HITO_CHARGING_INPUT_PIN, NRF_GPIO_PIN_PULLUP);
  }
}

void hito_led_on(hito_led_t led) {
  hito_led_off();
  switch(led) {
    case HITO_LED_RED:
      nrf_gpio_pin_clear(HITO_LED_RED_PIN);
      break;
    case HITO_LED_GREEN:
      nrf_gpio_pin_clear(legacy_bootloader 
          ? HITO_LED_BLUE_PIN : HITO_LED_GREEN_PIN);
      break;
    case HITO_LED_BLUE:
      nrf_gpio_pin_clear(legacy_bootloader 
          ? HITO_LED_GREEN_PIN : HITO_LED_BLUE_PIN);
      break;
    case HITO_LED_LCD:
      nrf_gpio_pin_set(HITO_ILI9342_LED_PIN);
      break;
    case HITO_LED_DEBUG:
      hito_debug_led_on();
      break;
    default: 
      break;
  }
}

void hito_led_off_one(hito_led_t led) {
  switch(led) {
    case HITO_LED_RED:
      nrf_gpio_pin_set(HITO_LED_RED_PIN);
      break;
    case HITO_LED_GREEN:
      nrf_gpio_pin_set(legacy_bootloader 
          ? HITO_LED_BLUE_PIN : HITO_LED_GREEN_PIN);
      break;
    case HITO_LED_BLUE:
      nrf_gpio_pin_set(legacy_bootloader 
          ? HITO_LED_GREEN_PIN : HITO_LED_BLUE_PIN);
      break;
    case HITO_LED_LCD:
      nrf_gpio_pin_clear(HITO_ILI9342_LED_PIN);
      break;
    case HITO_LED_DEBUG:
      hito_debug_led_off();
      break;
    default: 
      break;
  }
}

void hito_led_off() {
  nrf_gpio_pin_set(HITO_LED_RED_PIN);
  nrf_gpio_pin_set(HITO_LED_GREEN_PIN);
  nrf_gpio_pin_set(HITO_LED_BLUE_PIN);
  nrf_gpio_pin_set(HITO_LED_DEBUG_PIN);
  nrf_gpio_pin_clear(HITO_ILI9342_LED_PIN);
  hito_debug_led_off();
}

//-----------------------------------------------------------------------------
void hito_debug_led_blink(char n) {
  k_msleep(200);
  while(n-- > 0) {
    hito_debug_led_on();
    //hito_led_on(HITO_LED_RED);
    k_msleep(100);
    hito_debug_led_off();
    //hito_led_off(HITO_LED_RED);
    k_msleep(500);
  }
  k_msleep(1000);
}

//-----------------------------------------------------------------------------
bool hito_dev_board() {
  return (nrf_gpio_pin_read(HITO_DEVBOARD_DETECT_PIN) == 1) ? true : false;
}

