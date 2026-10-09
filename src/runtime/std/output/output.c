#include "runtime.h"
#include <stdint.h>
#include <stdio.h>
#include "pal.h"

void *dasu(int64_t value) {
    char buffer[32];
    int length = snprintf(buffer, sizeof(buffer), "%lld\n", (long long)value);
    if (length > 0) (void)pal_stdio_write(1, buffer, length);
    return NULL;
}

int64_t rt_io_write(int64_t selector, const char *data) {
    return rt_io_write_n(selector, data, -1);
}

int64_t rt_io_write_n(int64_t selector, const char *data, int64_t count) {
    if (data == NULL) return -1;
    size_t len = rt_managed_len(data);
    if (count >= 0 && (size_t)count < len) len = (size_t)count;
    if (len == 0) return 0;
    int64_t written = pal_stdio_write(selector, data, (int64_t)len);
    return written == (int64_t)len ? written : -1;
}

int64_t rt_io_flush(int64_t selector) {
    (void)selector;
    return 0;
}

int64_t rt_io_write_line(int64_t selector, const char *data) {
    if (data == NULL) return -1;
    size_t len = rt_managed_len(data);
    if (pal_stdio_write(selector, data, (int64_t)len) != (int64_t)len) return -1;
    if (pal_stdio_write(selector, "\n", 1) != 1) return -1;
    return (int64_t)(len + 1);
}

int64_t rt_io_write_i64(int64_t selector, int64_t value, int64_t newline) {
    char buf[32];
    int length = snprintf(buf, sizeof(buf), "%lld", (long long)value);
    if (length < 0) return -1;
    size_t byte_count = (size_t)length;
    if (newline && byte_count < sizeof(buf)) buf[byte_count++] = '\n';
    int64_t written = pal_stdio_write(selector, buf, (int64_t)byte_count);
    return written == (int64_t)byte_count ? written : -1;
}

int64_t rt_io_write_f64(int64_t selector, double value, int64_t newline) {
    char buf[64];
    int length = snprintf(buf, sizeof(buf), "%g", value);
    if (length < 0) return -1;
    size_t byte_count = (size_t)length;
    if (newline && byte_count < sizeof(buf)) buf[byte_count++] = '\n';
    int64_t written = pal_stdio_write(selector, buf, (int64_t)byte_count);
    return written == (int64_t)byte_count ? written : -1;
}
