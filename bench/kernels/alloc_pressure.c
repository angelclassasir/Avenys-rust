/* C2: allocation pressure.
 *
 * The C leg is the reference for what "equivalent" means. C has no language
 * runtime, so this kernel calls the C library directly: malloc, realloc, free,
 * and no arena, no reference counting, and no allocator of its own. A Mire
 * version that needed runtime machinery to match these numbers would be paying
 * for something C does not pay for, and the comparison would measure the runtime
 * rather than the language.
 *
 * The vector is a bare {int64 *data; size_t len, cap;} — the shape a C program
 * writes when it wants a growable array and nothing else. That is the shape the
 * Mire `vec[i64]` is measured against.
 *
 * OPACUE() is replaced by the harness with a heap-address-derived value, which
 * is what stops the optimizer from folding the loop into closed form. Replacing
 * it with a constant made two earlier measurements worthless.
 */
#include "_common.h"

int main(void) {
    int64_t acc = SEED;
    for (int64_t i = 0; i < 8000000; i++) {
        /* Empty creation, then eight pushes — the same sequence the Mire and
         * Rust legs perform. Initial capacity 8 so the C version does not
         * reallocate mid-loop; the growth path is measured by L2. */
        size_t cap = 8;
        int64_t *data = (int64_t *)bench_alloc_barrier(malloc(cap * sizeof(int64_t)),
                                                        cap * sizeof(int64_t));
        size_t len = 0;
        for (int64_t j = 0; j < 8; j++) {
            if (len == cap) {
                cap *= 2;
                data = (int64_t *)realloc(data, cap * sizeof(int64_t));
            }
            data[len++] = j + i;
        }
        acc = (acc + (int64_t)len + data[0]) % 1000003;
        free(data);
    }
    printf("%lld\n", (long long)acc);
    return 0;
}