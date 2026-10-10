/* P2: one trait, two implementations.
 *
 * C has no traits, so this leg dispatches through a function pointer held in a
 * vtable-shaped struct. That is C's closest equivalent, and the difference
 * between it and Mire's branch-by-tag dispatch is part of what this kernel
 * measures.
 *
 * The concrete type is carried alongside the dispatch rather than the struct
 * being reinterpreted through the vtable pointer: casting one struct to another
 * and reading a field through it is undefined behaviour, and it produced a
 * different result before it was separated here.
 */
#include "_common.h"

typedef struct { int64_t lado; } Cuadro;
typedef struct { int64_t radio; } Circulo;

__attribute__((noinline)) static int64_t valor_cuadro(int64_t lado) { return lado * lado; }
__attribute__((noinline)) static int64_t valor_circulo(int64_t radio) { return radio * radio * 3; }

typedef int64_t (*ValorFn)(int64_t);

int main(void) {
    int64_t acc = SEED;
    for (int64_t i = 0; i < 8000000; i++) {
        int64_t arg = (acc + i) % 1000003;
        ValorFn valor = (acc % 2) == 0 ? valor_cuadro : valor_circulo;
        acc = (acc + valor(arg)) % 1000003;
    }
    printf("%lld\n", (long long)acc);
    return 0;
}
