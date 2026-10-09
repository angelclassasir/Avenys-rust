#include "runtime.h"
#include <stdio.h>
#include <stdint.h>

static FILE *io_stream(int64_t selector) {
    return (selector == 2) ? stderr : stdout;
}

int64_t rt_io_error(int64_t selector) {
    return ferror(io_stream(selector)) ? 1 : 0;
}

int64_t rt_io_clear(int64_t selector) {
    clearerr(io_stream(selector));
    return 0;
}

int64_t rt_io_fd(int64_t selector) {
    return (selector == 2) ? 2 : 1;
}
