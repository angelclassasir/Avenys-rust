#include "runtime.h"
#include <stdint.h>
#include <stdlib.h>
#include <errno.h>
#include <string.h>
#include "pal.h"

char *rt_io_read_line(void);

char *ireru(const char *prompt) {
    if (prompt && *prompt) {
        (void)pal_stdio_write(1, prompt, (int64_t)strlen(prompt));
    }
    // Share the read buffer with the stdin APIs below. Mixing getchar() with
    // descriptor-level reads can hide buffered bytes from rt_io_available().
    return rt_io_read_line();
}

// stdin is read through the raw descriptor with a buffer of our own. This is
// deliberate: stdio can read ahead into an inaccessible buffer, causing
// rt_io_available() to report no data even though the process already holds it.
#define MIRE_IO_READ_BUF 8192

static char g_read_buf[MIRE_IO_READ_BUF];
static size_t g_read_len = 0;
static size_t g_read_pos = 0;
static int g_read_eof = 0;

static int64_t io_input_available(void) {
    if (g_read_pos < g_read_len) return 1;
    if (g_read_eof) return 0;
    return pal_stdin_available() > 0 ? 1 : 0;
}

static size_t io_read_fill(void) {
    if (g_read_pos < g_read_len) return g_read_len - g_read_pos;
    if (g_read_eof) return 0;
    g_read_pos = 0;
    g_read_len = 0;
    for (;;) {
        int64_t got = pal_stdin_read(g_read_buf, sizeof(g_read_buf));
        if (got > 0) {
            g_read_len = (size_t)got;
            return g_read_len;
        }
        if (got == 0) {
            g_read_eof = 1;
            return 0;
        }
        if (errno == EINTR) continue;
        g_read_eof = 1;
        return 0;
    }
}

int64_t rt_io_read_char(void) {
    if (io_read_fill() == 0) return -1;
    return (int64_t)(unsigned char)g_read_buf[g_read_pos++];
}

static int io_read_peek(const char **out, size_t *out_len) {
    if (io_read_fill() == 0) {
        *out = NULL;
        *out_len = 0;
        return 0;
    }
    *out = g_read_buf + g_read_pos;
    *out_len = g_read_len - g_read_pos;
    return 1;
}

char *rt_io_read_line(void) {
    size_t cap = 128, len = 0;
    char *buf = (char *)malloc(cap);
    if (!buf) return rt_managed_from_slice("", 0);
    const char *chunk;
    size_t chunk_len;
    while (io_read_peek(&chunk, &chunk_len)) {
        const char *nl = (const char *)memchr(chunk, '\n', chunk_len);
        size_t take = nl ? (size_t)(nl - chunk) : chunk_len;
        if (len + take + 1 > cap) {
            while (len + take + 1 > cap) cap *= 2;
            char *nb = (char *)realloc(buf, cap);
            if (!nb) break;
            buf = nb;
        }
        memcpy(buf + len, chunk, take);
        len += take;
        g_read_pos += take + (nl ? 1 : 0);
        if (nl) break;
    }
    if (len > 0 && buf[len - 1] == '\r') len--;
    char *result = rt_managed_from_slice(buf, len);
    free(buf);
    return result;
}

char *rt_io_read_bytes(int64_t count) {
    if (count <= 0) return rt_managed_from_slice("", 0);
    char *buf = (char *)malloc((size_t)count);
    if (!buf) return rt_managed_from_slice("", 0);
    size_t total = 0;
    while (total < (size_t)count) {
        const char *chunk;
        size_t chunk_len;
        if (!io_read_peek(&chunk, &chunk_len)) break;
        size_t take = chunk_len;
        if (take > (size_t)count - total) take = (size_t)count - total;
        memcpy(buf + total, chunk, take);
        total += take;
        g_read_pos += take;
    }
    char *result = rt_managed_from_slice(buf, total);
    free(buf);
    return result;
}

char *rt_io_read_all(void) {
    size_t cap = 8192, len = 0;
    char *buf = (char *)malloc(cap);
    if (!buf) return rt_managed_from_slice("", 0);
    for (;;) {
        const char *chunk;
        size_t chunk_len;
        if (!io_read_peek(&chunk, &chunk_len)) break;
        if (len + chunk_len + 1 > cap) {
            while (len + chunk_len + 1 > cap) cap *= 2;
            char *nb = (char *)realloc(buf, cap);
            if (!nb) break;
            buf = nb;
        }
        memcpy(buf + len, chunk, chunk_len);
        len += chunk_len;
        g_read_pos += chunk_len;
    }
    char *result = rt_managed_from_slice(buf, len);
    free(buf);
    return result;
}

int64_t rt_io_available(void) {
    return io_input_available();
}

int64_t rt_io_is_tty(void) {
    return pal_stdin_is_tty() > 0 ? 1 : 0;
}
