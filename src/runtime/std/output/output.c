#include "runtime.h"
#include <stdio.h>
#include <stdint.h>

static FILE *io_stream(int64_t selector) {
    return (selector == 2) ? stderr : stdout;
}

void *dasu(int64_t value) {
    printf("%ld\n", value);
    fflush(stdout);
    return NULL;
}

int64_t rt_io_write(int64_t selector, const char *data) {
    return rt_io_write_n(selector, data, -1);
}

int64_t rt_io_write_n(int64_t selector, const char *data, int64_t count) {
    if (data == NULL) return -1;
    FILE *stream = io_stream(selector);
    size_t len = rt_managed_len(data);
    if (count >= 0 && (size_t)count < len) len = (size_t)count;
    if (len == 0) {
        fflush(stream);
        return 0;
    }
    size_t written = fwrite(data, 1, len, stream);
    fflush(stream);
    if (written != len) return -1;
    return (int64_t)written;
}

int64_t rt_io_flush(int64_t selector) {
    return fflush(io_stream(selector)) == 0 ? 0 : -1;
}

int64_t rt_io_write_line(int64_t selector, const char *data) {
    if (data == NULL) return -1;
    FILE *stream = io_stream(selector);
    size_t len = rt_managed_len(data);
    if (fwrite(data, 1, len, stream) != len) return -1;
    if (fputc('\n', stream) == EOF) return -1;
    fflush(stream);
    return (int64_t)(len + 1);
}

int64_t rt_io_write_i64(int64_t selector, int64_t value, int64_t newline) {
    char buf[32];
    int length = snprintf(buf, sizeof(buf), "%lld", (long long)value);
    if (length < 0) return -1;
    size_t byte_count = (size_t)length;
    if (newline && byte_count < sizeof(buf)) buf[byte_count++] = '\n';
    FILE *stream = io_stream(selector);
    size_t written = fwrite(buf, 1, byte_count, stream);
    fflush(stream);
    return written == byte_count ? (int64_t)byte_count : -1;
}

int64_t rt_io_write_f64(int64_t selector, double value, int64_t newline) {
    char buf[64];
    int length = snprintf(buf, sizeof(buf), "%g", value);
    if (length < 0) return -1;
    size_t byte_count = (size_t)length;
    if (newline && byte_count < sizeof(buf)) buf[byte_count++] = '\n';
    FILE *stream = io_stream(selector);
    size_t written = fwrite(buf, 1, byte_count, stream);
    fflush(stream);
    return written == byte_count ? (int64_t)byte_count : -1;
}
