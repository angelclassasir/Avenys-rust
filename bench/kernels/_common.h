/* Shared preamble for the C benchmark legs.
 *
 * C is the reference leg for these kernels. It has no language runtime, so the
 * C versions reach the C library directly and use nothing else: no arena, no
 * reference counting, no allocator of its own. A Mire version that needed
 * runtime machinery to match these numbers would be measuring the runtime
 * rather than the language, which is the whole point of having this leg.
 *
 * Every kernel here contains an optimization barrier, and each one was added
 * because its absence produced a wrong measurement rather than a wrong guess:
 *
 *   - A literal seed lets the optimizer fold an accumulator into closed form.
 *     Two measurements were worthless before the barrier existed.
 *   - An allocation whose block never escapes is removed outright: the
 *     optimizer computes the stored values and drops the allocation. That made
 *     the allocation leg report a time no program performing eight million real
 *     malloc/free pairs could produce.
 *
 * The Rust and Mire legs substitute the same two seeds at their own build step,
 * so all three legs compare on identical work.
 */
#ifndef MIRE_BENCH_COMMON_H
#define MIRE_BENCH_COMMON_H

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

/* Verify builds use a fixed seed so all three legs can be compared; measure
 * builds use one derived from the address of a fresh allocation. */
#ifdef MIRE_BENCH_VERIFY
#define SEED ((int64_t)987654321)
#else
static inline int64_t bench_opaque_seed(void) {
    void *p = malloc(sizeof(int));
    int64_t value = (int64_t)(intptr_t)p;
    free(p);
    return value;
}
#define SEED bench_opaque_seed()
#endif

/* Barrier for allocated memory. Defined in both modes: a verify build still
 * has to perform the allocations it is verifying. */
__attribute__((noinline)) static void *bench_alloc_barrier(void *p, size_t n) {
    /* Read and write one byte, which the optimizer cannot prove away. */
    if (p && n) { ((volatile char *)p)[0] = ((volatile char *)p)[0]; }
    return p;
}

#endif /* MIRE_BENCH_COMMON_H */