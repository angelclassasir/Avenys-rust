/* A4: sorting.
 *
 * The C leg is the reference for what "equivalent" means. It uses an insertion
 * sort with the same in-place, contiguous layout the Mire runtime uses, so the
 * two legs compare the same algorithm on the same memory traffic. Using qsort
 * here would measure the C library's sort, not the language.
 */
#include "_common.h"

int main(void) {
    int64_t seed = SEED;
    for (int64_t round = 0; round < 200; round++) {
        int64_t v[2000];
        for (int64_t k = 0; k < 2000; k++) {
            v[k] = (k * 2654435761LL + round) % 1000003;
        }
        for (int64_t i = 1; i < 2000; i++) {
            int64_t key = v[i];
            int64_t j = i - 1;
            while (j >= 0 && v[j] > key) { v[j + 1] = v[j]; j--; }
            v[j + 1] = key;
        }
        seed = (seed + v[0] + v[1999]) % 1000003;
    }
    printf("%lld\n", (long long)seed);
    return 0;
}
