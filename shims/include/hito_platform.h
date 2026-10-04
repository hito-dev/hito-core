#ifndef __hito_platform_h_included__
#define __hito_platform_h_included__

#include <stdint.h>

int64_t hito_platform_time_uptime_ms(void);
void hito_platform_time_sleep_ms(uint32_t ms);

void hito_platform_log(const char *msg, uint16_t msg_len);

#endif//__hito_platform_h_included__
