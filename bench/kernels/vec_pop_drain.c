/* L4: build then drain with pop.
 *
 * Popping past the end has to read as 0 and leave the length alone rather than
 * computing len-1 and loading one word before the allocation. The C leg does it
 * the obvious way — check len, then return — so this kernel measures whether
 * the pop path is as cheap as the guard it has to perform.
 */
#include "_common.h"

int main(void) {
    int64_t seed = SEED;
    for (int64_t round = 0; round < 48000; round++) {
        size_t cap = 4;
        size_t len = 0;
        int64_t *data = (int64_t *)bench_alloc_barrier(malloc(cap * sizeof(int64_t)),
                                                        cap * sizeof(int64_t));
        for (int64_t k = 0; k < 200; k++) {
            if (len == cap) {
                size_t new_cap = cap < 4 ? 4 : cap + (cap >> 1);
                data = (int64_t *)realloc(data, new_cap * sizeof(int64_t));
                cap = new_cap;
            }
            data[len++] = k * 7 + round;
        }
        seed = (seed + (int64_t)len) % 1000003;
        for (int64_t j = 0; j < 200; j++) {
            int64_t popped = 0;
            if (len > 0) {
                popped = data[len - 1];
                len--;
            }
            seed = (seed + popped + j) % 1000003;
        }
        seed = (seed + (int64_t)len) % 1000003;
        free(data);
    }
    printf("%lld\n", (long long)seed);
    return 0;
}