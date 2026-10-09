#include "runtime.h"
#include "pal.h"
#include <stdint.h>

void rt_panic_loc(const char *msg, int64_t line, int64_t col, const char *file) {
    rt_runtime_write_panic_location(msg, line, col, file);
    pal_process_exit(1);
}

int64_t rt_div_i64(int64_t a, int64_t b, int64_t line, int64_t col, const char *file) {
    if (b == 0) rt_panic_loc("division by zero", line, col, file);
    if (a == INT64_MIN && b == -1) rt_panic_loc("integer overflow in division", line, col, file);
    return a / b;
}

int64_t rt_rem_i64(int64_t a, int64_t b, int64_t line, int64_t col, const char *file) {
    if (b == 0) rt_panic_loc("division by zero", line, col, file);
    if (a == INT64_MIN && b == -1) rt_panic_loc("integer overflow in remainder", line, col, file);
    return a % b;
}

void rt_check_bounds_i64(int64_t index, int64_t len, int64_t line, int64_t col, const char *file) {
    if (index < 0 || index >= len) rt_panic_loc("index out of bounds", line, col, file);
}

void *rt_closure_env_alloc(int64_t size) {
    if (size <= 0) return NULL;
    return pal_alloc(size);
}

void rt_closure_env_free(void *env) {
    if (env) pal_free(env);
}
