#include "../src/runtime/runtime.h"

#include <stdio.h>
#include <stdlib.h>

static int allocation_calls;
static int reallocation_calls;
static int release_calls;
static int outstanding_allocations;

void *pal_alloc(int64_t size) {
    if (size <= 0) return NULL;
    allocation_calls++;
    void *pointer = malloc((size_t)size);
    if (pointer) outstanding_allocations++;
    return pointer;
}

void *pal_realloc(void *pointer, int64_t size) {
    if (size <= 0) return NULL;
    reallocation_calls++;
    int was_unallocated = pointer == NULL;
    void *resized = realloc(pointer, (size_t)size);
    if (resized && was_unallocated) outstanding_allocations++;
    return resized;
}

void pal_free(void *pointer) {
    if (pointer) {
        release_calls++;
        outstanding_allocations--;
    }
    free(pointer);
}

int rt_managed_is_managed(const char *value) { (void)value; return 0; }
int rt_managed_contains(const char *value) { (void)value; return 0; }
MireManagedString *rt_string_header(const char *value) { (void)value; return NULL; }
char *rt_managed_from_slice(const char *value, size_t length) {
    (void)value;
    (void)length;
    return NULL;
}
void rt_managed_free(char *value) { (void)value; }
char *rt_strdup_raw(const char *value) { (void)value; return NULL; }
char *rt_alloc_printf_raw_i64(const char *format, long long value) {
    (void)format;
    (void)value;
    return NULL;
}

int main(void) {
    void *map = NULL;
    for (int64_t index = 0; index < 128; index++) {
        map = rt_dict_set_i64(map, MIRE_KIND_SCALAR, MIRE_KIND_SCALAR,
                              index, NULL, index * 3);
        for (int64_t checked = 0; checked <= index; checked++) {
            if (rt_dict_get_i64(map, MIRE_KIND_SCALAR, checked, NULL, -1) != checked * 3) {
                fprintf(stderr, "lookup failed after insert %lld for key %lld\n",
                        (long long)index, (long long)checked);
                return 1;
            }
        }
    }

    if (allocation_calls < 2 || reallocation_calls < 3) {
        fprintf(stderr, "map storage bypassed PAL allocators: alloc=%d realloc=%d\n",
                allocation_calls, reallocation_calls);
        return 1;
    }

    rt_dict_free(map);
    if (outstanding_allocations != 0) {
        fprintf(stderr, "map storage release mismatch: remaining=%d alloc=%d realloc=%d free=%d\n",
                outstanding_allocations, allocation_calls, reallocation_calls, release_calls);
        return 1;
    }

    puts("Map container storage uses PAL allocation, growth, and release.");
    return 0;
}
