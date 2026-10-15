/* L3: vector growth.
 *
 * The counterpart to C2. That kernel preallocates with_capacity(8) and pushes
 * exactly eight times, so it never reallocates; this one starts empty and grows
 * to 2000, reallocating about nine times per vector. It is the leg that measures
 * what a growing vector costs, which C2 deliberately avoids.
 *
 * Growth is 1.5x (matching the runtime's rt_list_create/list_grow), so the
 * allocation is reallocated a handful of times and each reallocation copies
 * everything written so far.
 */
#include "_common.h"

int main(void) {
    int64_t seed = SEED;
    for (int64_t round = 0; round < 44000; round++) {
        size_t cap = 4;
        size_t len = 0;
        int64_t *data = (int64_t *)bench_alloc_barrier(malloc(cap * sizeof(int64_t)),
                                                        cap * sizeof(int64_t));
        for (int64_t k = 0; k < 2000; k++) {
            if (len == cap) {
                size_t new_cap = cap < 4 ? 4 : cap + (cap >> 1);
                data = (int64_t *)realloc(data, new_cap * sizeof(int64_t));
                cap = new_cap;
            }
            data[len++] = k * 3 + round;
        }
        seed = (seed + (int64_t)len + data[1999]) % 1000003;
        free(data);
    }
    printf("%lld\n", (long long)seed);
    return 0;
}