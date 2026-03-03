#include <hito_platform.h>
#include <zephyr.h>

// Rust shim to get system uptime in milliseconds - orig k_uptime_get is inline 
int64_t hito_platform_time_uptime_ms(void)
{
    return k_uptime_get();
}

// Rust shim to sleep for a number of milliseconds - orig k_msleep is inline
void hito_platform_time_sleep_ms(uint32_t ms)
{
    k_msleep(ms);
}
