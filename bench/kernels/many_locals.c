/* V1: 23 live locals, register pressure. */
#include "_common.h"

int main(void) {
    int64_t a01=1,a02=2,a03=3,a04=4,a05=5,a06=6,a07=7,a08=8;
    int64_t a09=9,a10=10,a11=11,a12=12,a13=13,a14=14,a15=15,a16=16;
    int64_t a17=17,a18=18,a19=19,a20=20,a21=21,a22=22,a23=23;
    int64_t acc = SEED;
    for (int64_t i = 0; i < 4000000; i++) {
        acc = (acc+a01+a02+a03+a04+a05+a06+a07+a08) % 1000003;
        acc = (acc+a09+a10+a11+a12+a13+a14+a15+a16) % 1000003;
        acc = (acc+a17+a18+a19+a20+a21+a22+a23) % 1000003;
        acc = acc + i;
    }
    printf("%lld\n", (long long)acc);
    return 0;
}
