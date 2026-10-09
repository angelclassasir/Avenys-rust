#include "runtime.h"
#include <stdint.h>
#include <stdio.h>
#include <string.h>
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

static void write_diagnostic_text(const char *text) {
    if (text) (void)pal_stdio_write(2, text, (int64_t)strlen(text));
}

void rt_runtime_write_panic_location(const char *message, int64_t line,
                                     int64_t column, const char *file) {
    char position[64];
    if (file && file[0]) {
        write_diagnostic_text(file);
        write_diagnostic_text(":");
        int length = snprintf(position, sizeof(position), "%lld:%lld: ",
                              (long long)line, (long long)column);
        if (length > 0) (void)pal_stdio_write(2, position, length);
    } else if (line > 0 || column > 0) {
        int length = snprintf(position, sizeof(position), "%lld:%lld: ",
                              (long long)line, (long long)column);
        if (length > 0) (void)pal_stdio_write(2, position, length);
    }
    write_diagnostic_text(message ? message : "unknown error");
    write_diagnostic_text("\n");
}

void rt_runtime_write_panic_message(const char *message, int use_unknown_fallback) {
    write_diagnostic_text("runtime error");
    if (message && message[0]) {
        write_diagnostic_text(": ");
        write_diagnostic_text(message);
    } else if (use_unknown_fallback) {
        write_diagnostic_text(": unknown");
    }
    write_diagnostic_text("\n");
}
