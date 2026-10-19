/* A5: 2D access pattern.
 *
 * A flat vector filled row-major, then summed column-major so the second pass
 * strides across cache lines. The C leg mirrors the same row-major fill and
 * column-major accumulation on a plain heap array.
 */
#include "_common.h"

int main(void) {
    int64_t seed = SEED;
    int64_t n = 128;
    for (int64_t round = 0; round < 800; round++) {
        int64_t *m = (int64_t *)malloc((size_t)(n * n * sizeof(int64_t)));
        if (!m) return 1;
        for (int64_t r = 0; r < n; r++) {
            for (int64_t c = 0; c < n; c++) {
                m[r * n + c] = r * n + c + round;
            }
        }
        int64_t acc = 0;
        for (int64_t c = 0; c < n; c++) {
            for (int64_t r = 0; r < n; r++) {
                acc += m[r * n + c];
            }
        }
        free(m);
        seed = (seed + acc) % 1000003;
    }
    printf("%lld\n", (long long)seed);
    return 0;
}
