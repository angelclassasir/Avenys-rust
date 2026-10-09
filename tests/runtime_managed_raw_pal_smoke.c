#include "../src/runtime/runtime.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static int allocation_calls;
static int release_calls;

void *pal_alloc(int64_t size) {
    if (size <= 0) return NULL;
    allocation_calls++;
    return malloc((size_t)size);
}

void pal_free(void *pointer) {
    if (pointer) release_calls++;
    free(pointer);
}

int main(void) {
    char *copy = rt_strdup_raw("runtime");
    char *slice = rt_strdup_raw_n("abcdef", 3);
    char *formatted = rt_alloc_printf_raw_i64("value=%lld", -42);
    if (!copy || !slice || !formatted || strcmp(copy, "runtime") != 0 ||
        strcmp(slice, "abc") != 0 || strcmp(formatted, "value=-42") != 0) {
        fprintf(stderr, "raw string PAL allocation returned incorrect content\n");
        return 1;
    }

    pal_free(copy);
    pal_free(slice);
    pal_free(formatted);
    if (allocation_calls != 3 || release_calls != 3) {
        fprintf(stderr, "raw string PAL allocation mismatch: alloc=%d free=%d\n",
                allocation_calls, release_calls);
        return 1;
    }

    puts("Full-runtime raw string helpers use matched PAL allocation and release.");
    return 0;
}
