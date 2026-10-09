#include "runtime.h"
#include <stdint.h>
#include "pal.h"

int64_t rt_io_error(int64_t selector) {
    return pal_stdio_error(selector);
}

int64_t rt_io_clear(int64_t selector) {
    return pal_stdio_clear_error(selector);
}

int64_t rt_io_fd(int64_t selector) {
    return (selector == 2) ? 2 : 1;
}
