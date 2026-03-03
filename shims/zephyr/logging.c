#include <hito_platform.h>

#include <zephyr.h>
//#include <logging/log.h>
//LOG_MODULE_REGISTER(hito_platform, LOG_LEVEL_INF);

// Rust shim to sleep for a number of milliseconds - orig k_msleep is inline
void hito_platform_log(const char * msg, uint16_t msg_len)
{
    printk("%.*s", (size_t)msg_len, msg);
    //LOG_INF("%.*s", (size_t)msg_len, msg);
    //LOG_RAW(LOG_LEVEL_NONE, "%.*s", (size_t)msg_len, msg);
}
