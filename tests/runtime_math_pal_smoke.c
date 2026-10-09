#include "../src/runtime/runtime.h"

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

typedef struct {
    const int64_t *values;
    int64_t length;
} test_vector_t;

static int64_t allocation_calls;
static int64_t release_calls;

void *pal_alloc(int64_t size) {
    if (size <= 0) return NULL;
    allocation_calls++;
    return malloc((size_t)size);
}

void pal_free(void *pointer) {
    if (pointer) release_calls++;
    free(pointer);
}

int64_t rt_list_len(void *list) {
    return list ? ((test_vector_t *)list)->length : 0;
}

int64_t rt_list_get_i64(void *list, int64_t index) {
    test_vector_t *vector = list;
    if (!vector || index < 0 || index >= vector->length) return 0;
    return vector->values[index];
}

int main(void) {
    const int64_t odd_values[] = {9, 1, 5};
    const int64_t even_values[] = {8, 2, 6, 4};
    test_vector_t odd = {odd_values, 3};
    test_vector_t even = {even_values, 4};

    double odd_median = rt_math_median_i64(&odd);
    double even_median = rt_math_median_i64(&even);
    if (odd_median != 5.0 || even_median != 5.0 ||
        allocation_calls != 2 || release_calls != 2) {
        fprintf(stderr,
                "median PAL smoke failed: odd=%g even=%g alloc=%lld free=%lld\n",
                odd_median, even_median, (long long)allocation_calls,
                (long long)release_calls);
        return 1;
    }
    puts("Runtime median temporary buffers use the PAL allocator.");
    return 0;
}
