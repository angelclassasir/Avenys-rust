/* P3: a method delegating through a trait to another type. */
#include "_common.h"

typedef struct { int64_t a, b; } Rect;
typedef struct { int64_t a; } Cuad;

__attribute__((noinline)) static int64_t area_rect(Rect *s) { return s->a * s->b; }
__attribute__((noinline)) static int64_t area_cuad(Cuad *s) { return s->a * s->a; }

typedef struct { int64_t alto, ancho; } Figura;

__attribute__((noinline)) static int64_t superficie(Figura *f) {
    if (f->alto == f->ancho) { Cuad c; c.a = f->alto; return area_cuad(&c); }
    Rect r; r.a = f->alto; r.b = f->ancho; return area_rect(&r);
}

int main(void) {
    int64_t acc = SEED;
    for (int64_t i = 0; i < 8000000; i++) {
        Figura f; f.alto = acc % 97; f.ancho = acc % 97 + 1;
        acc = (acc + superficie(&f)) % 1000003;
    }
    printf("%lld\n", (long long)acc);
    return 0;
}
