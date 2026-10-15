/* L5: mixed reads over one resident vector.
 *
 * The vector is built once and stays resident, so this kernel measures the read
 * path rather than the allocator. The index steps by a stride coprime with 4096,
 * which sweeps the whole array on every pass instead of revisiting one cache
 * line — the point is to keep the reads honest.
 */
#include "_common.h"

int main(void) {
    int64_t seed = SEED;
    size_t cap = 4096;
    size_t len = 0;
    int64_t *data = (int64_t *)bench_alloc_barrier(malloc(cap * sizeof(int64_t)),
                                                    cap * sizeof(int64_t));
    for (int64_t k = 0; k < 4096; k++) {
        data[len++] = k * 5 + 1;
    }
    for (int64_t j = 0; j < 12000000; j++) {
        int64_t idx = (j * 4093) % 4096;
        seed = (seed + data[idx] + (int64_t)len) % 1000003;
    }
    seed = (seed + data[0] + data[4095]) % 1000003;
    free(data);
    printf("%lld\n", (long long)seed);
    return 0;
}