/* P1: struct method call.
 *
 * `noinline` on the method mirrors the Rust and Mire legs, which both mark the
 * method so the call is real. Without it the call is folded away and the kernel
 * measures nothing.
 */
#include "_common.h"

typedef struct { int64_t n; } Contador;

__attribute__((noinline)) static int64_t total(Contador *self) { return self->n * 2 + 1; }

int main(void) {
    int64_t acc = SEED;
    for (int64_t i = 0; i < 20000000; i++) {
        Contador c; c.n = (acc + i) % 1000003;
        acc = (acc + total(&c)) % 1000003;
    }
    printf("%lld\n", (long long)acc);
    return 0;
}
