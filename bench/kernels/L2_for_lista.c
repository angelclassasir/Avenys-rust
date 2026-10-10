/* L2: indexed reads over a dynamic array.
 *
 * The array is a bare malloc'd buffer: what a C program writes when it wants a
 * growable array and nothing else. That is the shape `vec[i64]` is measured
 * against.
 */
#include "_common.h"

int main(void) {
    size_t cap = 2048, len = 0;
    int64_t *v = (int64_t *)malloc(cap * sizeof(int64_t));
    for (int64_t i = 0; i < 2000; i++) {
        if (len == cap) { cap *= 2; v = (int64_t *)realloc(v, cap * sizeof(int64_t)); }
        v[len++] = i * 3;
    }
    int64_t acc = SEED;
    for (int64_t j = 0; j < 4000000; j++) { acc = acc % 1000003 + v[j % 2000]; }
    printf("%lld\n", (long long)acc);
    free(v);
    return 0;
}
