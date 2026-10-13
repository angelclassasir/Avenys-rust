/* C3: integer and float interleaved. */
#include "_common.h"

int main(void) {
    int64_t acc = SEED;
    double f = 1.5;
    for (int64_t i = 0; i < 8000000; i++) {
        acc = (acc + (i * 3) / 7 - i % 5) % 1000003;
        f = f * 1.0000001 + (double)(acc % 7);
        if (i % 1000 == 0) { f /= 1.0000001; }
        acc += i % 3;
    }
    printf("%lld\n", (long long)acc);
    printf("%s\n", f > 0.0 ? "true" : "false");
    return 0;
}
