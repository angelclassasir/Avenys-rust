/* A8: partitioning a vector into two by a predicate.
 *
 * Each round fills a source vector, then drains it into an even and an odd
 * vector, so the timed work is one input plus two output allocations per round.
 */
#include "_common.h"

int main(void) {
    int64_t seed = SEED;
    for (int64_t round = 0; round < 40000; round++) {
        int64_t src[128];
        int64_t even[128], odd[128];
        int elen = 0, olen = 0;
        for (int64_t k = 0; k < 128; k++) src[k] = k * 7 + round;
        for (int64_t j = 0; j < 128; j++) {
            int64_t x = src[j];
            if (x % 2 == 0) even[elen++] = x;
            else odd[olen++] = x;
        }
        seed = (seed + elen + olen) % 1000003;
    }
    printf("%lld\n", (long long)seed);
    return 0;
}
