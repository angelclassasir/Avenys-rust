/* L3: two levels of nested loops. */
#include "_common.h"

int main(void) {
    int64_t acc = SEED;
    for (int64_t i = 0; i < 8000; i++) {
        for (int64_t j = 0; j < 8000; j++) { acc += (i ^ j) % 11; }
    }
    printf("%lld\n", (long long)acc);
    return 0;
}
