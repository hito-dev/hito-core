#include "hito_button.h"
#include "hito_pin_config.h"
#include "hito_power.h"

#include <device.h>
#include <hal/nrf_gpio.h>
#include <drivers/gpio.h>

bool hito_button_is_pressed() {
  return nrf_gpio_pin_read(HITO_BUTTON_PIN) == 0 ? true : false;
}

uint32_t hito_button_press_time; 

static void hito_button_handler(const struct device *dev, 
    struct gpio_callback *cb, uint32_t pins)
{
  //ili9342_lcd_led_off();
  //ft6336_ctp_power_off();
  printk("button is pressed: %s\n", hito_button_is_pressed() ? "true" : "false");

  if (hito_button_is_pressed()) {
    hito_button_press_time = k_uptime_get_32();
  } else {
    if (k_uptime_get_32() - hito_button_press_time < 500) {
      printk("power off by button press\n");
      hito_power_off();
    }
  }

  /*
  if (k_uptime_get_32() - hito_button_init_time > 1000) {
    printk("power off by button press\n");
    while(hito_button_is_pressed()) {
      //printk("button is pressed\n");
      k_msleep(250);
    }
    printk("button is not pressed, power off\n");
    k_msleep(1000);
    hito_power_off();
  }
  */
}

void hito_button_init() 
{

  const struct device * gpio_dev = device_get_binding(DT_LABEL(DT_NODELABEL(gpio0)));
	if (gpio_dev == NULL) {
		printk("GPIO_0 bind error");
		return;
	}

  static struct gpio_callback ctp_cb;

  gpio_pin_configure(gpio_dev, HITO_BUTTON_PIN,
                     GPIO_INPUT | GPIO_PULL_UP);
  gpio_pin_interrupt_configure(gpio_dev, 
      HITO_BUTTON_PIN, GPIO_INT_EDGE_BOTH);

  gpio_init_callback(&ctp_cb, 
      hito_button_handler, BIT(HITO_BUTTON_PIN));
  //
  gpio_add_callback(gpio_dev, &ctp_cb);

}


