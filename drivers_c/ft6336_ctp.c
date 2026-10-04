#include "ft6336_ctp.h"
#include "hito_pin_config.h"

#include <nrfx_gpiote.h>
#include <device.h>
#include <drivers/i2c.h>
#include <drivers/gpio.h>
#include <hal/nrf_gpio.h>

#define I2C_ADDRESS 0x38
//#define I2C_ADDRESS  DT_PROP(DT_ALIAS(hito_ctp), reg);

// Touch x
uint16_t m_touch_x;

// Touch y
uint16_t m_touch_y;

// Indicates that 
bool     m_has_touch;

// First 
uint32_t m_touch_time;
uint32_t m_last_touch_time;

uint32_t m_touch_count = 0;

//-----------------------------------------------------------------------------
// forward declarations

bool ctp_sleep_mode = false;

void ft6336_i2c_send(uint8_t reg, uint8_t val);

//-----------------------------------------------------------------------------
//void ft6336_ctp_interrupt_handler(nrfx_gpiote_pin_t pin, nrf_gpiote_polarity_t action)
static void ft6336_ctp_interrupt_handler(const struct device *dev, struct gpio_callback *cb,
                           uint32_t pins)
{
  m_touch_count++;

  m_touch_time = k_uptime_get_32();
  m_has_touch = true;

}

uint32_t ft6336_ctp_touch_count() {
  return m_touch_count;
}

void ft6336_ctp_imitate_touch() {
  m_last_touch_time = k_uptime_get_32();
}

const struct device *i2c_dev;
const struct device *gpio_dev;

//-----------------------------------------------------------------------------
void ft6336_ctp_init_twi() {

	i2c_dev = device_get_binding(DT_LABEL(DT_ALIAS(hito_ctp)));
	if (!i2c_dev) {
		printk("I2C: Device driver not found.\n");
		return;
	}
  i2c_configure(i2c_dev, I2C_SPEED_SET(I2C_SPEED_STANDARD));


  // send init to ft3603u
  //uint8_t reg[2];
  ft6336_i2c_send(0x00, 0x00); // period monitor, 0x80 ms
  ft6336_i2c_send(0x88, 0x03); // period active , 0x01
  // ft6336_i2c_send(0x80, 0x2e); // period active , 0x01
  // ft6336_i2c_send(0x86, 0xd9); // period active , 0x01
  ft6336_i2c_send(0x86, 0x00); // always stay in active
  // ft6336_i2c_send(0xa4, 0x00); // interrupt polling mode
  printk("i2c init completed\n");
}

const struct device *gpio_dev;
static struct gpio_callback ctp_cb;
//-----------------------------------------------------------------------------
void ft6336_ctp_init_interrupt() 
{
  gpio_dev = device_get_binding(DT_LABEL(DT_NODELABEL(gpio0)));
	if (gpio_dev == NULL) {
		printk("GPIO_0 bind error");
		return;
	}


  gpio_pin_configure(gpio_dev, HITO_FT6306_INT_PIN,
                     GPIO_INPUT | GPIO_PULL_UP);
  gpio_pin_interrupt_configure(gpio_dev, 
      HITO_FT6306_INT_PIN, GPIO_INT_EDGE_BOTH);

  gpio_init_callback(&ctp_cb, 
      ft6336_ctp_interrupt_handler, BIT(HITO_FT6306_INT_PIN));
  //
  gpio_add_callback(gpio_dev, &ctp_cb);

}

void ft6336_ctp_uninit() {
  gpio_remove_callback(gpio_dev, &ctp_cb);
  gpio_pin_configure(gpio_dev, HITO_FT6306_INT_PIN, GPIO_OUTPUT);
}

/*
void ft6336_ctp_uninit() {
  nrfx_gpiote_in_event_enable(m_ctp_config.interrupt, false);
  nrfx_gpiote_in_event_enable(HITO_CHARGING_INPUT_PIN, false);
  nrf_drv_gpiote_uninit();
}
*/


void ft6336_ctp_touch_wait(void (*idle)()) {
  //nrf_drv_twi_uninit(&m_twi);
  //nrf_gpio_pin_set(m_ctp_config.reset);

  m_has_touch = false;
  while(!m_has_touch) {
    if (idle) { 
      idle();
    } else {
      k_msleep(25);
    }
  }

  //NRF_LOG_INFO("dddd"); NRF_LOG_FLUSH();

  //uint8_t reg[2];
  //reg[0] = 0xA5; // device mode
  //reg[1] = 0x03; // monitor
  //ret_code_t err_code;
  //err_code = nrf_drv_twi_tx(&m_twi, m_ctp_config.i2c_address, reg, sizeof(reg), false);
  //APP_ERROR_CHECK(err_code);
  ////nrfx_gpiote_in_event_enable(m_ctp_config.interrupt, false);
  ////nrfx_gpiote_uninit();
  ////nrf_drv_twi_uninit(&m_twi);
  //nrf_gpio_pin_clear(m_ctp_config.reset);
  //nrf_gpio_pin_clear(m_ctp_config.power);
  ////nrf_gpio_pin_clear(m_ctp_config.sda);
  ////nrf_gpio_pin_clear(m_ctp_config.scl);
  ////nrf_gpio_cfg_output(m_ctp_config.interrupt);
  ////nrf_gpio_pin_clear(m_ctp_config.interrupt);
  //////nrf_drv_twi_disable(&m_twi);
  //nrf_delay_ms(100);
  //nrf_gpio_pin_set(m_ctp_config.power);
  //nrf_delay_ms(50);
  //nrf_gpio_pin_set(m_ctp_config.reset);
  //nrf_delay_ms(2);
  //nrf_gpio_pin_clear(m_ctp_config.reset);
  //nrf_delay_ms(2);
  ////nrf_gpio_pin_set(m_ctp_config.reset);
  //nrf_delay_ms(100);
  //ft6336_ctp_init_twi();
  //ft6336_ctp_init_interrupt();
  //
  //nrf_gpio_pin_set(m_ctp_config.reset);
  //nrf_delay_ms(2);
  //nrf_gpio_pin_clear(m_ctp_config.reset);
  //nrf_delay_ms(2);

  //NRF_LOG_INFO("eeee"); NRF_LOG_FLUSH();
}

void ft6336_i2c_send(uint8_t reg, uint8_t val) 
{
	struct i2c_msg msg;

	/* Setup I2C messages */
	uint8_t wr_addr[2];

	/* Send the address to read from */
	msg.buf = wr_addr;

	msg.len = 2;
	msg.flags = I2C_MSG_WRITE | I2C_MSG_STOP;

  wr_addr[0] = reg;
  wr_addr[1] = val;

  int ret = i2c_transfer(i2c_dev, &msg, 1, I2C_ADDRESS);
  if (ret) {
    printk("Error reading from FT6306 error code (%d)\n", ret);
  }
}

//-----------------------------------------------------------------------------
void ft6336_ctp_init() {

  // init data
  m_touch_time = 0;
  m_last_touch_time  = 0;
  m_has_touch        = false;

  /*
  nrf_gpio_cfg(
      m_ctp_config.power,
      NRF_GPIO_PIN_DIR_OUTPUT,
      NRF_GPIO_PIN_INPUT_DISCONNECT,
      NRF_GPIO_PIN_NOPULL,
      NRF_GPIO_PIN_H0H1,
      NRF_GPIO_PIN_NOSENSE);

  //nrf_gpio_pin_clear(m_ctp_config.power);
  //nrf_delay_ms(10);
  nrf_gpio_pin_set(m_ctp_config.power);

  nrf_delay_ms(50);

  nrf_gpio_cfg_output(m_ctp_config.reset);
  //nrf_gpio_pin_set(m_ctp_config.reset);
  nrf_delay_ms(2);
  nrf_gpio_pin_clear(m_ctp_config.reset);
  nrf_delay_ms(2);
  nrf_gpio_pin_set(m_ctp_config.reset);

  ft6336_ctp_init_twi();
  hito_debug_led_off();
  //nrf_drv_twi_disable(&m_twi);
  //nrf_drv_twi_uninit(&m_twi);
  //nrf_gpio_pin_clear(m_ctp_config.sda);
  //nrf_gpio_pin_clear(m_ctp_config.scl);
  */

  ft6336_ctp_init_interrupt();
}

//-----------------------------------------------------------------------------
void ft6336_ctp_power_on() {
  nrf_gpio_pin_set(HITO_FT6306_POWER_PIN);

  nrf_gpio_pin_clear(HITO_FT6306_RESET_PIN);
  k_msleep(1);
  nrf_gpio_pin_set(HITO_FT6306_RESET_PIN);
  k_msleep(150);

  ft6336_ctp_init_twi();
}

//-----------------------------------------------------------------------------
void ft6336_ctp_power_off() {

  nrf_gpio_pin_clear(HITO_FT6306_POWER_PIN);
  nrf_gpio_pin_clear(HITO_FT6306_RESET_PIN);
}

//-----------------------------------------------------------------------------
bool ft6336_ctp_is_touch_released() {
  if (k_uptime_get_32() - m_touch_time > 50) {
    m_has_touch = false;
    return true;
  } else {
    return false;
  }
}

//-----------------------------------------------------------------------------
bool ft6336_ctp_is_pressed() {
  if (k_uptime_get_32() - m_touch_time > 50) {
    return false;
  } else {
    return true;
  }
}

void ft6336_ctp_power_reset() 
{
  nrf_gpio_pin_clear(HITO_FT6306_POWER_PIN);
  k_msleep(1);
  nrf_gpio_pin_set(HITO_FT6306_POWER_PIN);
  k_msleep(125);

  ft6336_i2c_send(0x88, 0x03);
}

//-----------------------------------------------------------------------------
bool ft6336_ctp_read_touch() 
{
	struct i2c_msg msgs[2];

	/* Setup I2C messages */
	uint8_t wr_addr[2];
	uint8_t buf[4];
  int touch_count;
	uint8_t data[16];

	/* Send the address to read from */
	msgs[0].buf = wr_addr;

	msgs[0].len = 1;
	msgs[0].flags = I2C_MSG_WRITE | I2C_MSG_STOP;

	/* Read from device. STOP after this. */
	msgs[1].buf = data;
	msgs[1].flags = I2C_MSG_READ | I2C_MSG_STOP;

  if (!m_has_touch) {
    return false;
  }

  m_has_touch = false;

  // read number of touch
  wr_addr[0] = 0x02;
  msgs[1].len = 1;
  data[0] = 0;
  int ret = i2c_transfer(i2c_dev, &msgs[0], 2, I2C_ADDRESS);
  if (ret) {
    printk("Error reading from FT6306 error code (%d)\n", ret);
    return false;
  } else if (data[0] >= 1) {
    touch_count = data[0];
    wr_addr[0] = 0x03;
    msgs[1].len = 4;
    data[0] = data[1] = data[2] = data[3] = 0;
    ret = i2c_transfer(i2c_dev, &msgs[0], 2, I2C_ADDRESS);
    // printk("touch_count: %d\n", touch_count);
    // printk("1: %x %x %x %x\n", data[0], data[1], data[2], data[3]);
    m_touch_x = ((data[0] & 0x0f) << 8) + data[1];
    // printk("x before: %d", m_touch_x);
    double x = m_touch_x;
    m_touch_x = -13.6130873481304 + 1.03180366959475 * x -0.00157472407213897 * x * x + 0.0000201591939009656 * x * x * x -0.000000047145716844327 * x * x * x * x;
    // printk(" x after: %d\n", m_touch_x);
    m_touch_y = ((data[2] & 0x0f) << 8) + data[3];
    return true;
  }
  return false;
}

//-----------------------------------------------------------------------------
inline bool ft6336_ctp_has_touch() {
  if (m_has_touch) {
    return true;
  } else {
    return false;
  }
}

//-----------------------------------------------------------------------------
inline bool touch_released() {
  return m_has_touch;
}

//-----------------------------------------------------------------------------
inline uint16_t ft6336_ctp_touch_x() {
  return m_touch_x;
}

//-----------------------------------------------------------------------------
inline uint16_t ft6336_ctp_touch_y() {
  return m_touch_y;
}

//-----------------------------------------------------------------------------
inline uint32_t ft6336_ctp_last_touch_time() {
  return m_last_touch_time;
}

//-----------------------------------------------------------------------------
inline uint32_t ft6336_ctp_first_touch_time() {
  return m_touch_time;
}



