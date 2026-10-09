#include "../src/runtime/runtime.h"

#include <stdio.h>
#include <string.h>

static int64_t current_cpu_time_ms;
static char formatted_elapsed[32];

int64_t pal_cpu_time_ms(void) { return current_cpu_time_ms; }

char *rt_managed_printf_i64(const char *format, long long value) {
    snprintf(formatted_elapsed, sizeof(formatted_elapsed), format, value);
    return formatted_elapsed;
}

int main(void) {
    current_cpu_time_ms = 987;
    char *elapsed = rt_cpu_elapsed_ms_str(900);
    if (!elapsed || strcmp(elapsed, "87") != 0) {
        fprintf(stderr, "CPU elapsed time mismatch: expected 87 ms, got %s\n",
                elapsed ? elapsed : "<null>");
        return 1;
    }

    puts("CPU elapsed-time formatting uses the PAL millisecond clock.");
    return 0;
}
