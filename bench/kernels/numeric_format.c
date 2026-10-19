/* A9: number-to-text and back.
 *
 * Formats an integer to a buffer and parses it back each round, exercising the
 * integer<->string round trip on both legs.
 */
#include "_common.h"
#include <stdio.h>

int main(void) {
    int64_t seed = SEED;
    char buf[32];
    for (int64_t i = 0; i < 2000000; i++) {
        int64_t n = (seed + i) % 1000000;
        int len = snprintf(buf, sizeof(buf), "%lld", (long long)n);
        int64_t back = 0;
        sscanf(buf, "%lld", (long long *)&back);
        seed = (seed + back + (int64_t)len) % 1000003;
    }
    printf("%lld\n", (long long)seed);
    return 0;
}
