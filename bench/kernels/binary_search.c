/* A6: binary search over a sorted vector.
 *
 * Builds a sorted vector once per round, then probes it 256 times. The C leg
 * mirrors the same binary search so both legs exercise identical comparisons
 * and branch patterns.
 */
#include "_common.h"

int main(void) {
    int64_t seed = SEED;
    int64_t n = 4096;
    int64_t *v = (int64_t *)malloc((size_t)(n * sizeof(int64_t)));
    if (!v) return 1;
    for (int64_t round = 0; round < 2000; round++) {
        for (int64_t k = 0; k < n; k++) v[k] = k * 2;
        for (int64_t probe = 0; probe < 256; probe++) {
            int64_t target = (probe * 7 + round) % 8192;
            int64_t lo = 0, hi = n - 1, idx = -1;
            while (lo <= hi) {
                int64_t mid = lo + (hi - lo) / 2;
                if (v[mid] == target) { idx = mid; break; }
                if (v[mid] < target) lo = mid + 1;
                else hi = mid - 1;
            }
            seed = (seed + idx + probe) % 1000003;
        }
    }
    free(v);
    printf("%lld\n", (long long)seed);
    return 0;
}
