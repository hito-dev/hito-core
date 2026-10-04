#include <zephyr.h>
#include <kernel.h>

int64_t rust_k_uptime_get(void) {
    return k_ticks_to_ns_floor64(k_uptime_ticks());
}