/* C1: ackermann(3, 10). */
#include "_common.h"

__attribute__((noinline)) static int64_t ack(int64_t m, int64_t n) {
    if (m == 0) return n + 1;
    if (n == 0) return ack(m - 1, 1);
    return ack(m - 1, ack(m, n - 1));
}

int main(void) { printf("%lld\n", (long long)ack(3, 10)); return 0; }
