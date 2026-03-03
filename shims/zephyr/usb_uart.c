#include <zephyr.h>
#include <device.h>
#include <drivers/uart.h>
#include <usb/usb_device.h>

const struct device *hito_platform_device_get_binding(const char *name) {
    return device_get_binding(name);
}

int hito_platform_usb_enable() {
    return usb_enable(NULL);
}


int hito_platform_uart_poll_in(const struct device *dev, uint8_t *c) {
    return uart_poll_in(dev, c);
}

void hito_platform_uart_poll_out(const struct device *dev, uint8_t c) {
    uart_poll_out(dev, c);
}

int hito_platform_uart_line_ctrl_get(const struct device *dev, uint32_t ctrl, uint32_t *val) {
#if defined(CONFIG_UART_LINE_CTRL)
    return uart_line_ctrl_get(dev, ctrl, val);
#else
    ARG_UNUSED(dev); ARG_UNUSED(ctrl); ARG_UNUSED(val);
    return -ENOTSUP;
#endif
}

int hito_platform_uart_line_ctrl_set(const struct device *dev, uint32_t ctrl, uint32_t val) {
#if defined(CONFIG_UART_LINE_CTRL)
    return uart_line_ctrl_set(dev, ctrl, val);
#else
    ARG_UNUSED(dev); ARG_UNUSED(ctrl); ARG_UNUSED(val);
    return -ENOTSUP;
#endif
}

