/* V2: 8-way rotation driven by the accumulator. */
#include "_common.h"

int main(void) {
    int64_t a=1,b=2,c=3,d=4,e=5,f=6,g=7,h=8;
    int64_t acc = SEED;
    for (int64_t i = 0; i < 6000000; i++) {
        int64_t t = a + acc % 3;
        a=b; b=c; c=d; d=e; e=f; f=g; g=h; h=t;
        acc = (acc + a - h) % 1000003;
    }
    printf("%lld\n", (long long)acc);
    return 0;
}
