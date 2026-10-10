/* L1: while loop with a counter. */
#include "_common.h"

int main(void) {
    int64_t acc = SEED;
    for (int64_t i = 0; i < 40000000; i++) { acc += i % 7; }
    printf("%lld\n", (long long)acc);
    return 0;
}
