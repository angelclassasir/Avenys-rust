#include "../src/runtime/runtime.h"

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static int64_t allocation_calls;
static int64_t reallocation_calls;
static int64_t release_calls;

void *pal_alloc(int64_t size) {
    if (size <= 0) return NULL;
    allocation_calls++;
    return malloc((size_t)size);
}

void *pal_realloc(void *pointer, int64_t size) {
    if (size <= 0) return NULL;
    reallocation_calls++;
    return realloc(pointer, (size_t)size);
}

void pal_free(void *pointer) {
    if (pointer) release_calls++;
    free(pointer);
}

int main(void) {
    char *first = rt_managed_from_slice("owned", 5);
    int first_ok = first && strcmp(first, "owned") == 0;
    rt_managed_free(first);

    char *second = rt_managed_from_cstr("again");
    int second_ok = second && strcmp(second, "again") == 0;
    rt_managed_free(second);

    int64_t calls_before_oversized_request = allocation_calls;
    char *oversized = rt_managed_alloc(SIZE_MAX);
    int oversized_rejected = oversized == NULL &&
                             allocation_calls == calls_before_oversized_request;

    char static_literal[] = "not owned by the runtime";
    int64_t releases_before_literal = release_calls;
    rt_managed_free(static_literal);
    int literal_preserved = release_calls == releases_before_literal &&
                            strcmp(static_literal, "not owned by the runtime") == 0;

    if (!first_ok || !second_ok || !oversized_rejected || !literal_preserved ||
        allocation_calls != 2 || reallocation_calls != 1 || release_calls != 2) {
        fprintf(stderr,
                "Minimal managed PAL smoke failed: first=%d second=%d size=%d literal=%d alloc=%lld realloc=%lld free=%lld\n",
                first_ok, second_ok, oversized_rejected, literal_preserved,
                (long long)allocation_calls, (long long)reallocation_calls,
                (long long)release_calls);
        return 1;
    }
    puts("Minimal managed strings preserve ownership and use PAL allocation.");
    return 0;
}
