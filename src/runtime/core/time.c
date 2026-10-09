#include "runtime.h"
#include "pal.h"

char *rt_time_elapsed_ms_str(int64_t start_ns) {
    int64_t now_ns = pal_time_monotonic_ns();
    int64_t elapsed_ms = now_ns < start_ns ? 0 : (now_ns - start_ns) / 1000000;
    return rt_managed_printf_i64("%lld", (long long)elapsed_ms);
}
