/* L4: continue in a loop.
 *
 * Written as a while loop with the increment in the body, matching the Rust and
 * Mire legs. A `for` loop with an extra `i++` in the body advances twice per
 * iteration and halves the work, which showed up as a different result.
 */
#include "_common.h"

int main(void) {
    int64_t acc = SEED;
    int64_t i = 0;
    while (i < 20000000) {
        i++;
        if (i % 3 == 0) { continue; }
        if (i % 1000 == 0) { acc += i; }
        if (i % 2 == 0) { acc -= 1; } else { acc += 2; }
    }
    printf("%lld\n", (long long)acc);
    return 0;
}
