#include "ili9342_lcd.h"
#include "hito_pin_config.h"


#include <errno.h>
#include <zephyr.h>
#include <sys/printk.h>
#include <device.h>
#include <drivers/spi.h>
#include <drivers/gpio.h>
#include <nrfx_spim.h>
#include <string.h>
#include <hal/nrf_gpio.h>
#include <drivers/led.h>

#define ILI9342_CASET       0x2A
#define ILI9342_PASET       0x2B
#define ILI9342_RAMWR       0x2C
#define ILI9342_RAMRD       0x2E

#define max(a,b) a > b ? a : b
#define min(a,b) a < b ? a : b

//-----------------------------------------------------------------------------
// variables and constants
// TODO optimize to SCREEN_WIDTH * 2 or even smaller buffer with multiple transfers
#define      ILI9342_TX_BUF_SIZE 15360
//#define      ILI9342_TX_BUF_SIZE 7680
static uint8_t      m_tx_buf[ILI9342_TX_BUF_SIZE];

// TODO get rid of this big buffer and use smaller one with multiple transfers
//static uint8_t buf[320*240*2];
static uint8_t buf[2];

#if DT_NODE_HAS_STATUS(DT_INST(0, pwm_leds), okay)
#define LED_PWM_NODE_ID  DT_INST(0, pwm_leds)
#define LED_PWM_DEV_NAME DEVICE_DT_NAME(LED_PWM_NODE_ID)
#else
#error "No LED PWM device found"
#endif
//const struct device *spi;
//const struct device *gpio_dev;

//struct spi_config spi_cfg = {0};

//-----------------------------------------------------------------------------
// forward declarations
void ili9342_lcd_write_command(const char * cmd);

void ili9342_lcd_set_addr_window(uint16_t x_0, uint16_t y_0, 
                                       uint16_t x_1, uint16_t y_1);

static const nrfx_spim_t m_nrfx_spim = NRFX_SPIM_INSTANCE(4);
nrfx_spim_xfer_desc_t m_xfer_desc;

//-----------------------------------------------------------------------------
void ili9342_lcd_spi_init() {

	//spi = device_get_binding(DT_LABEL(DT_ALIAS(hito_lcd)));
  //xfer_desc.
  m_xfer_desc.p_tx_buffer = m_tx_buf;
  m_xfer_desc.tx_length   = 0;
  m_xfer_desc.p_rx_buffer = NULL;
  m_xfer_desc.rx_length   = 0;

	//if (!spi) {
	//	printk("Could not find SPI driver\n");
	//}

  //// init spi
	//spi_cfg.operation = SPI_WORD_SET(8);
	////spi_cfg.operation = SPI_WORD_SET(8);
	////spi_cfg.frequency = 32000000U;
	//spi_cfg.frequency = 16000000U;
  //printk("miso_pin: %d\nmosi_pin: %d\nsck_pin:  %d\ncs_pin:   %d\n", 
  //    0,//spi_cfg.miso_pin,
  //    0,//spi_cfg.mosi_pin,
  //    0,//spi_cfg.sck_pin,
  //    spi_cfg.cs->gpio_pin
  //);
  nrfx_spim_config_t spi_config = NRFX_SPIM_DEFAULT_CONFIG(
      HITO_ILI9342_SCK_PIN,
      HITO_ILI9342_MOSI_PIN,
      NRFX_SPIM_PIN_NOT_USED,
      HITO_ILI9342_CS_PIN
  );
  //spi_config.frequency      = NRF_SPIM_FREQ_16M;
  spi_config.frequency      = NRF_SPIM_FREQ_16M;
  //spi_config.frequency      = NRF_SPIM_FREQ_4M;
  //spi_config.ss_pin         = NRFX_SPIM_PIN_NOT_USED;
  spi_config.ss_pin         = HITO_ILI9342_CS_PIN;
  spi_config.miso_pin       = NRFX_SPIM_PIN_NOT_USED;
  //spi_config.miso_pin       = HITO_NAND_MISO_PIN;
  spi_config.mosi_pin       = HITO_ILI9342_MOSI_PIN;
  spi_config.sck_pin        = HITO_ILI9342_SCK_PIN;
  spi_config.dcx_pin        = HITO_ILI9342_DC_PIN;
  spi_config.rx_delay       = 1;
  spi_config.use_hw_ss      = true;
  spi_config.ss_active_high = false;
  //  ////spi_config.rx_delay = 10;
  //err_code = nrfx_spim_init(&spi, &spi_config, spim_event_handler, NULL);

  nrfx_spim_uninit(&m_nrfx_spim);
  int err_code = nrfx_spim_init(&m_nrfx_spim, &spi_config, NULL, NULL);
  if (err_code != NRFX_SUCCESS) {
    printk("nrfx_spim_init %x\n", err_code);
  }
  printk("nrfx_spim_init %x\n", err_code);

}

//-----------------------------------------------------------------------------
void ili9342_lcd_power_on()
{

  nrf_gpio_pin_set(HITO_ILI9342_POWER_PIN);
  k_msleep(100);
  ili9342_lcd_reset();

  // #TRICK disable touchscreen interference
  //ft6336_ctp_power_reset();
};

//-----------------------------------------------------------------------------
void ili9342_lcd_power_off()
{
  nrf_gpio_pin_clear(HITO_ILI9342_POWER_PIN);
  nrf_gpio_pin_clear(HITO_ILI9342_RESET_PIN);
};

//int ili9342_hardware_init() {
  //int ret;

  //hito_pin_config();

  // init spi
//	spi_cfg.operation = SPI_WORD_SET(8);
//	//spi_cfg.operation = SPI_WORD_SET(8);
//	//spi_cfg.frequency = 64000000U;
//	spi_cfg.frequency = 16000000U;
//  printk("miso_pin: %d\nmosi_pin: %d\nsck_pin:  %d\ncs_pin:   %d\n", 
//      0,//spi_cfg.miso_pin,
//      0,//spi_cfg.mosi_pin,
//      0,//spi_cfg.sck_pin,
//      spi_cfg.cs->gpio_pin
//  );
//
//  //nrf_gpio_cfg(
//  //    HITO_ILI9342_POWER_PIN,
//  //    NRF_GPIO_PIN_DIR_OUTPUT,
//  //    NRF_GPIO_PIN_INPUT_DISCONNECT,
//  //    NRF_GPIO_PIN_PULLDOWN,
//  //    NRF_GPIO_PIN_S0H1,
//  //    NRF_GPIO_PIN_NOSENSE);
//
//  //nrf_gpio_pin_set(HITO_ILI9342_POWER_PIN);
//
//
//  return ret;
//}
//
int ili9342_display_init();

//-----------------------------------------------------------------------------
void ili9342_lcd_init() {

  ili9342_lcd_spi_init();
  
  //int err = ili9342_hardware_init();
  //if (err) {
  //  printk("Could not init ili9342 hardware\n");
  //}

  ili9342_display_init();
  //ili9342_lcd_power_on();
  //ili9342_lcd_reset();

  //ili9342_lcd_led_on();
}

//struct spi_buf spi_bufs[1];
//struct spi_buf_set spi_tx;


// send first byte as command 
void write_command(const char * cmd) {

  //write_command_n_data(cmd, 1, 0);
  //if (strlen(cmd) > 1) {
  //  write_command_n_data(cmd + 1, 0, strlen(cmd) - 1);
  //}
  m_xfer_desc.p_tx_buffer = m_tx_buf;
  m_xfer_desc.tx_length   = strlen(cmd);
  memcpy(m_tx_buf, cmd, m_xfer_desc.tx_length);
  m_xfer_desc.p_rx_buffer = NULL;
  m_xfer_desc.rx_length   = 0;
  //const char * p = cmd;
  //NRF_LOG_INFO("spi_send_command:    %d", xfer_desc.tx_length);
  //NRF_LOG_HEXDUMP_INFO(xfer_desc.p_tx_buffer, xfer_desc.tx_length);

  //nrfx_spim_xfer_desc_t data = NRFX_SPIM_XFER_TX(cmd, strlen(cmd));

  uint8_t cmd_len = m_xfer_desc.tx_length == 1 ? NRF_SPIM_DCX_CNT_ALL_CMD : 1;
  uint32_t err_code;
  err_code = nrfx_spim_xfer_dcx(&m_nrfx_spim, &m_xfer_desc, 0, cmd_len);

}

int ili9342_display_init() {

  nrf_gpio_pin_clear(HITO_ILI9342_POWER_PIN);
  //nrf_gpio_pin_clear(ILI9342_MOSI_PIN);
  //nrf_gpio_cfg_output(ILI9342_POWER_PIN);
  //gpio_pin_set(gpio_dev, ILI9342_RESET_PIN, 0);
  nrf_gpio_pin_clear(HITO_ILI9342_RESET_PIN);
  k_msleep(100);
  nrf_gpio_pin_set(HITO_ILI9342_POWER_PIN);
  k_msleep(100);
  // reset device
  //gpio_pin_set(gpio_dev, ILI9342_RESET_PIN, 1);
  nrf_gpio_pin_set(HITO_ILI9342_RESET_PIN);
  k_msleep(100);
  //k_msleep(250);
  //
  //nrfx_spim_config_t nrfx_spim_config = NRFX_SPIM_DEFAULT_CONFIG(
  //    ILI9342_SCK_PIN,
  //    ILI9342_MOSI_PIN,
  //    0x00,
  //    ILI9342_CS_PIN
  //);
  //nrfx_spim_config.frequency      = NRF_SPIM_FREQ_16M;
  ////nrfx_spim_config.ss_pin         = NRFX_SPIM_PIN_NOT_USED;
  //nrfx_spim_config.ss_pin         = ILI9342_CS_PIN;
  //nrfx_spim_config.miso_pin       = NRFX_SPIM_PIN_NOT_USED;
  //nrfx_spim_config.mosi_pin       = ILI9342_MOSI_PIN;
  //nrfx_spim_config.sck_pin        = ILI9342_SCK_PIN;
  //nrfx_spim_config.dcx_pin        = ILI9342_DC_PIN;
  //nrfx_spim_config.rx_delay       = 1;
  //nrfx_spim_config.use_hw_ss      = true;
  //nrfx_spim_config.ss_active_high = false;
  //nrfx_spim_init(&nrfx_spim, &nrfx_spim_config, NULL, NULL);


  /* startup sequence for MI0283QT-9A */
  write_command("\x01"); /* software reset */
  k_msleep(20);
  write_command("\x28"); /* display off */

  k_msleep(20);
  //write_command("\xB9\xFF\x93\x42"); // EXTC Enabled
  //write_command("\x36\xC8"); // Memory access control
  //write_command("\xC0\x28\x0A"); // Power control voltage for Gamma
  //write_command("\xC1\x02");     // Power control
  //write_command("\xC5\x45");      // VCOM Control
  //write_command("\xC7\xC3");      // GPIO Status
  ////write_command("\xB8\xFF"); // Backlight
  //write_command("\xB8\x0B"); // Backlight
  //write_command("\x51\x00"); // Brightness
  //write_command("\x3A\x55"); // COLMOD Color Mode 565

  //// Gamma positive and negative
  //write_command("\xE0\x0F\x2A\x27\x0C\x0F\x07\x58\x86\x48\x09\x18\x0B\x1B\x0E\x08");
  //write_command("\xE1\x08\x17\x1A\x02\x0E\x03\x29\x13\x39\x01\x05\x03\x26\x33\x0F");
  //k_msleep(20);

  //write_command("\x11"); //Exit Sleep
  //k_msleep(50);
  //write_command("\x29"); //Display ON
  //k_msleep(20);
  write_command("\xC8\xFF\x93\x42");  // Set EXTC
  write_command("\x36\xC8");  // Memory Access Control MY,MX,MV,ML,BGR,MH
  write_command("\x3A\x55");  // Pixel Format Set DPI [2:0], DBI [2:0]
  write_command("\xC0\x14\x0E"); // Power control voltage for Gamma VRH[5:0] VC[3:0]
  write_command("\xC1\x01"); // Power control 2 SAP[2:0] BT[3:0]
  write_command("\xC5\xFA");      // VCOM Control

  write_command("\xB1\x00\x1B");
  write_command("\xB4\x02");
  write_command("\xE0\x00\x02\x09\x07\x14\x09\x39\x89\x48\x09\x0F\x09\x1B\x1B\x0F");
  write_command("\xE1\x00\x24\x26\x02\x0F\x06\x3A\x25\x4D\x03\x0D\x0C\x35\x37\x0F");

  write_command("\x11"); //Exit Sleep
  write_command("\x21"); //Inversion on
  k_msleep(120);
  write_command("\x29"); //Display ON
  k_msleep(20);

  printk("ili9342c init completed\n");
  return 0;
}

void ili9342_lcd_set_brightness(uint8_t brightness) {
  if (brightness <= 0) {
    brightness = 0;
  } else if (brightness >= 100) {
    brightness = 100;
  }
  const struct device * led_pwm = device_get_binding(LED_PWM_DEV_NAME);         
  led_set_brightness(led_pwm, 0, brightness);      
}


//void ili9342_lcd_reset() {
//  // reset device
//  nrf_gpio_pin_clear(HITO_ILI9342_RESET_PIN);
//  k_msleep(1);
//  nrf_gpio_pin_set(HITO_ILI9342_RESET_PIN);
//  k_msleep(10);
//
//  /* startup sequence for MI0283QT-9A */
//  write_command("\x01"); /* software reset */
//  k_msleep(20);
//  write_command("\x28"); /* display off */
//
//  k_msleep(20);
//
//  write_command("\xC8\xFF\x93\x42");  // Set EXTC
//  write_command("\x36\xC8");  // Memory Access Control MY,MX,MV,ML,BGR,MH
//  write_command("\x3A\x55");  // Pixel Format Set DPI [2:0], DBI [2:0]
//  write_command("\xC0\x14\x0E"); // Power control voltage for Gamma VRH[5:0] VC[3:0]
//  write_command("\xC1\x01"); // Power control 2 SAP[2:0] BT[3:0]
//  write_command("\xC5\xFA");      // VCOM Control
//
//  write_command("\xB1\x00\x1B");
//  write_command("\xB4\x02");
//  write_command_n_data("\xE0\x00\x02\x09\x07\x14\x09\x39\x89\x48\x09\x0F\x09\x1B\x1B\x0F", 1, 15);
//  write_command_n_data("\xE1\x00\x24\x26\x02\x0F\x06\x3A\x25\x4D\x03\x0D\x0C\x35\x37\x0F", 1, 15);
//
//  write_command("\x11"); //Exit Sleep
//  write_command("\x21"); //Inversion on
//  k_msleep(120);
//  write_command("\x29"); //Display ON
//  k_msleep(20);
//
//  printk("ili9342c init completed\n");
//}


//-----------------------------------------------------------------------------
void ili9342_lcd_led_off() {
  nrf_gpio_pin_clear(HITO_ILI9342_LED_PIN);
}

//-----------------------------------------------------------------------------
void ili9342_lcd_led_on() {
  nrf_gpio_pin_set(HITO_ILI9342_LED_PIN);
}

//-----------------------------------------------------------------------------
void ili9342_lcd_sleep_in() {
  k_msleep(10);
  write_command("\x10");
}

//-----------------------------------------------------------------------------
void ili9342_lcd_sleep_out() {
  //nrf_gpio_pin_set(m_ili9342_lcd_config.led);
  k_msleep(10);
  write_command("\x11");
  k_msleep(10);
}

//-----------------------------------------------------------------------------
void ili9342_lcd_display_on() {
  k_msleep(10);
  write_command("\x29");
  k_msleep(10);
}

//-----------------------------------------------------------------------------
void ili9342_lcd_display_off() {
  k_msleep(10);
  write_command("\x28");
}

//-----------------------------------------------------------------------------
void ili9342_lcd_fill_rect(uint16_t x, uint16_t y, 
    uint16_t width, uint16_t height, uint16_t color565) 
{
    ili9342_lcd_set_addr_window(x, y, x + width - 1, y + height - 1);

    const uint8_t data[2] = {color565 >> 8, color565};

    nrf_gpio_pin_set(HITO_ILI9342_DC_PIN);
    //k_usleep(2);

    // Duff's device algorithm for optimizing loop.
    uint32_t i = (height * width + 7) / 8;
    
    uint8_t * p = (uint8_t *)buf;
    switch ((height * width) % 8) {
        case 0:
            do {
                *p++ = data[0];
                *p++ = data[1];
        case 7:
                *p++ = data[0];
                *p++ = data[1];
        case 6:
                *p++ = data[0];
                *p++ = data[1];
        case 5:
                *p++ = data[0];
                *p++ = data[1];
        case 4:
                *p++ = data[0];
                *p++ = data[1];
        case 3:
                *p++ = data[0];
                *p++ = data[1];
        case 2:
                *p++ = data[0];
                *p++ = data[1];
        case 1:
                *p++ = data[0];
                *p++ = data[1];
            } while (--i > 0);
        default:
            break;
    }

    uint8_t * p2 = (uint8_t *)buf;
    //p2++;
    int n = 15360;
    //n/=2;

    while(p2 < p) {
      memcpy(m_tx_buf, p2, n);
      //spi_buf[0].buf = m_tx_buf;
      //spi_buf[0].len = (p - p2 > 15360) ? 15360 : p - p2;
      //write_command_n_data(m_tx_buf, 0, (p - p2 > n) ? n : p - p2);
      //xfer_desc.tx_length   = (p - p2 > 15360) ? 15360 : p - p2;
      //nrfx_spim_xfer_dcx(&nrfx_spi, &xfer_desc, 0, 0);
      m_xfer_desc.tx_length   = (p - p2 > 15360) ? 15360 : p - p2;
      nrfx_spim_xfer_dcx(&m_nrfx_spim, &m_xfer_desc, 0, 0);
      p2+=15360;
    }

    nrf_gpio_pin_clear(HITO_ILI9342_DC_PIN);
    //NRF_LOG_INFO("%d", p - buf);
}

void ili9342_lcd_draw_screen_corners(bool allCorners) {
  allCorners = true;
  uint16_t color = 0x0000;
  ili9342_lcd_fill_rect(0, 0, 2, 2, color);
  ili9342_lcd_fill_rect(2, 0, 2, 1, color);
  ili9342_lcd_fill_rect(0, 2, 1, 2, color);
  
  ili9342_lcd_fill_rect(318, 0, 2, 2, color);
  ili9342_lcd_fill_rect(316, 0, 2, 1, color);
  ili9342_lcd_fill_rect(319, 2, 1, 2, color);

  if (allCorners) {
    ili9342_lcd_fill_rect(318, 238, 2, 2, color);
    ili9342_lcd_fill_rect(316, 239, 2, 1, color);
    ili9342_lcd_fill_rect(319, 236, 1, 2, color);

    ili9342_lcd_fill_rect(0, 238, 2, 2, color);
    ili9342_lcd_fill_rect(2, 239, 2, 1, color);
    ili9342_lcd_fill_rect(0, 236, 1, 2, color);
  }
}

#if __BYTE_ORDER__ == __ORDER_BIG_ENDIAN__
    #define LCD_NEEDS_SWAP 0
#else
    #define LCD_NEEDS_SWAP 1
#endif

void ili9342_lcd_draw_line_buffer(uint16_t x, uint16_t y, uint16_t w, const uint8_t * buf) {
  ili9342_lcd_set_addr_window(x, y, x + w - 1, y);
  nrf_gpio_pin_set(HITO_ILI9342_DC_PIN);
  m_xfer_desc.tx_length   = w * 2;
  #if LCD_NEEDS_SWAP
    for(int i = 0; i < w * 2; i+=2) {
      m_tx_buf[i] = buf[i+1];
      m_tx_buf[i+1] = buf[i];
    }
  #else
    memcpy(m_tx_buf, buf, w * 2);
  #endif
  //for(int i = 0; i < w * 2; i+=2) {
  //  m_tx_buf[i] = 0x00;
  //  m_tx_buf[i+1] = 0x1f;
  //}
  //m_xfer_desc.p_tx_buffer = buf;
  nrfx_spim_xfer_dcx(&m_nrfx_spim, &m_xfer_desc, 0, 0);
  nrf_gpio_pin_clear(HITO_ILI9342_DC_PIN);
}

//-----------------------------------------------------------------------------
void ili9342_lcd_draw_bitmap(uint16_t x, uint16_t y, uint16_t w, uint16_t h,
  const uint8_t * bitmap, uint16_t bitmap_row_pitch)
{
  // rreturn ili9342_lcd_draw_rect(rect, color565);
  ili9342_lcd_set_addr_window(x, y, x + w - 1, y + h - 1);

  //nrf_gpio_pin_set(m_ili9342_lcd_config.dc);
  nrf_gpio_pin_set(HITO_ILI9342_DC_PIN);

  while(h-- > 0) {
    memcpy(m_tx_buf, bitmap, w * 2);
    m_xfer_desc.tx_length   = w * 2;
    nrfx_spim_xfer_dcx(&m_nrfx_spim, &m_xfer_desc, 0, 0);
    //write_command_n_data(m_tx_buf, 0, w * 2);
    bitmap += bitmap_row_pitch;
  }

  //nrf_gpio_pin_clear(m_ili9342_lcd_config.dc);
  nrf_gpio_pin_clear(HITO_ILI9342_DC_PIN);
}

bool ili9342_lcd_draw_qr(const uint8_t * qr_data, uint16_t x, uint16_t y, 
    const uint8_t qr_width, uint16_t image_max_width, bool center)
{
  if (qr_width > image_max_width) {
    //LOG_ERR("qr_width > image_max_width, %d > %d", qr_width, image_max_width);
    return false;
  }
  int qr_density = image_max_width / qr_width;
  int image_width = qr_width * qr_density;

  uint8_t * p = buf;
  for(int i = 0; i < image_width * image_width; i++) {
    int x = i / image_width / qr_density;
    int y = (i % image_width) / qr_density;
    uint8_t color = qr_data[x * qr_width + y] & 0x01 ? 0x00 : 0xff;
    //color = 0xaa;

    *p++ = color;
    *p++ = color;
  }

  ili9342_lcd_draw_bitmap(x + (image_max_width - image_width) / 2, 
                          y + (image_max_width - image_width) / 2, 
                          image_width, image_width, buf, image_width * 2);

  //for(int x = 0; x < qr_width; x++) {
    //for(int y = 0; y < qr_width; y++) {
      //uint8_t color = qr_data[x * qr_width + y] & 0x01 ? 0x00 : 0xff;
      //m_buf[x * qr_density ] = 0xff
    //}
  //}

  return true;
}


void ili9342_lcd_set_addr_window(uint16_t x_0, uint16_t y_0, uint16_t x_1, uint16_t y_1)
{
    //ASSERT(x_0 <= x_1);
    //ASSERT(y_0 <= y_1);

    //uint8_t buf[10];

    m_tx_buf[0] = ILI9342_CASET;
    m_tx_buf[1] = (x_0 >> 8);
    m_tx_buf[2] = x_0;
    m_tx_buf[3] = (x_1 >> 8);
    m_tx_buf[4] = x_1;
    m_xfer_desc.tx_length = 5;
    nrfx_spim_xfer_dcx(&m_nrfx_spim, &m_xfer_desc, 0, 1);
    //write_command_n_data(m_tx_buf, 1, 4);
    

    m_tx_buf[0] = ILI9342_PASET;
    m_tx_buf[1] = (y_0 >> 8);
    m_tx_buf[2] = y_0;
    m_tx_buf[3] = (y_1 >> 8);
    m_tx_buf[4] = y_1;
    nrfx_spim_xfer_dcx(&m_nrfx_spim, &m_xfer_desc, 0, 1);
    //write_command_n_data(m_tx_buf, 1, 4);

    m_tx_buf[0] = ILI9342_RAMWR;
    m_tx_buf[1] = 0;
    m_xfer_desc.tx_length = 1;
    nrfx_spim_xfer_dcx(&m_nrfx_spim, &m_xfer_desc, 0, 1);
    //write_command(m_tx_buf);
    //write_command((char *)buf);
}
