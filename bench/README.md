# bench/ — Avenys contra Rust sobre el mismo cálculo

Cada kernel existe dos veces, en `kernels/*.mire` y `kernels/*.rs`, y hace
**exactamente el mismo cálculo**. Eso es lo que permite afirmar algo: no se
compara "un programa de Mire" contra "un programa de Rust", se compara el
resultado de un compilador contra el resultado del otro sobre la misma entrada.

## Uso

```sh
./run.sh verify            # comprueba que los dos lenguajes coinciden
./run.sh measure           # mide los doce kernels
./run.sh measure C2_allocacion
./run.sh curve C2_allocacion   # barrido de -O0 a -Oz
./run.sh startup           # coste de arranque de un programa trivial
```

`verify` es un paso previo, no una formalidad: `measure` se niega a dar
números si algún kernel difiere, porque una comparación entre programas que no
hacen lo mismo no mide nada.

Variables de entorno: `MIRE_BENCH_WORK` (directorio de trabajo, por defecto
`/tmp/mire-bench`), `MIRE_BIN` (por defecto `~/.local/bin/mire`).

## Kernels

| Kernel | Qué mide | Dificultad |
|---|---|---|
| `L1_while` | bucle `while` con contador | trivial |
| `L2_for_lista` | lectura indexada sobre lista dinámica | fácil |
| `L3_anidados` | dos niveles de bucle anidados | media |
| `L4_break_continue` | `break` y `continue` | media |
| `V1_muchas_locales` | 24 locales vivos, presión de registro | media |
| `V2_intercambio` | rotación de 8 variables | fácil |
| `P1_metodo` | struct con método | trivial |
| `P2_trait_dos` | un skill, dos implementaciones, despacho por rama | media |
| `P3_cadena_impl` | método que delega vía skill a otro tipo | difícil |
| `C1_recursion` | ackermann(3, 10) | difícil |
| `C2_allocacion` | un `vec` nuevo por iteración, 8M de ellos | difícil |
| `C3_mixed_ops` | entero y flotante intercalados | media |

`noop` no es un kernel: sirve para `startup`.

## Resultado (rustc 1.99 `-C opt-level=3`, Mire `-O3`, mejor de 5)

| Kernel | Dificultad | Rust | Mire | Mire/Rust |
|---|---|---|---|---|
| C1_recursion | difícil | 35 ms | 29 ms | 0.83x |
| C2_allocacion | difficult | 26 ms | 1314 ms | **50.54x** |
| C3_mixed_ops | media | 38 ms | 38 ms | 1.00x |
| L1_while | trivial | 47 ms | 47 ms | 1.00x |
| L2_for_lista | fácil | 14 ms | 22 ms | 1.57x |
| L3_anidados | media | 50 ms | 53 ms | 1.06x |
| L4_break_continue | media | 29 ms | 25 ms | 0.86x |
| P1_metodo | trivial | 127 ms | 129 ms | 1.02x |
| P2_trait_dos | media | 58 ms | 59 ms | 1.02x |
| P3_cadena_impl | difícil | 56 ms | 56 ms | 1.00x |
| V1_muchas_locales | media | 44 ms | 44 ms | 1.00x |
| V2_intercambio | fácil | 34 ms | 34 ms | 1.00x |

The reading is unchanged for control flow, variables, and object-oriented
features: Mire is close to Rust within measurement noise. The current gap is
the collection allocator, not LLVM optimization.

## Curva de optimización (C2_allocacion)

| | -O0 | -O1 | -O2 | -O3 | -Os | -Oz |
|---|---|---|---|---|---|---|
| Rust | 825 ms | 114 ms | 26 ms | 26 ms | 126 ms | 128 ms |
| Mire, stabilized ownership baseline | pending | pending | pending | 1314 ms | pending | pending |

Before A1, Mire was near 1.5 s because vectors were sent through the string
free path and were never reused. The preliminary 81 ms result is invalidated:
it exposed a second ownership bug where `vec.push` returned storage that the
assignment cleanup immediately freed. After stabilizing that transfer and
cleaning repeated loop declarations, the honest current baseline is about
1314 ms, or 50.54x Rust. This is the allocator baseline for B6; it is no
longer hidden by undefined behavior.

### Resultado de A1

Con `verify` pasando los 12 kernels bit a bit y el mismo arnés:

| Estado | Rust -O3 | Mire -O3 | Mire/Rust |
|---|---:|---:|---:|
| Before A1 | 26 ms | 1481 ms | 56.96x |
| Preliminary A1 result, invalidated by the push ownership bug | 26 ms | 81 ms | 3.12x |
| Stabilized ownership baseline | 26 ms | 1314 ms | 50.54x |

## Por qué los kernels son como son

Esto costó dos mediciones falsas, así que queda escrito.

**La barrera de opacidad.** El acumulador de cada kernel se inicializa con
`SEED_OPACA`. En `verify` vale una constante, y las salidas se comparan. En
`measure` se sustituye por la dirección de una reserva de heap: no hace falta
que cambie entre corridas —ASLR puede estar desactivado— sino que el compilador
no pueda conocerla en tiempo de compilación. Sin esa barrera, LLVM resuelve
`acc = acc * 3 + 1` en forma cerrada y "mide" 1 ms por 50M de iteraciones, que
es imposible. En Rust el equivalente es `std::hint::black_box`.

**La dependencia serial.** El acumulador entra de vuelta en cada iteración. Con
`acc += i` y las variables invariantes, LLVM pliega el bucle entero y lo convierte
en `acc + K * n`. Los kernels que fallaron así eran los que "ganaban" 50x.

**El piso de ruido.** El arranque de un binario de Mire cuesta lo mismo que el
de Rust: ~1 ms. Cualquier kernel por debajo de ~25 ms está midiendo el
arranque. `measure` marca esos resultados con `<-- ruido` en vez de
presentarlos.

**Los argumentos llevan coma explícita.** `ack(m (n - 1))` se parsea como una
llamada a `m`: la coma entre argumentos es opcional, y un `(` tras una
expresión completa se lee como invocación de un valor-función. `ack(m, (n - 1))`
es lo correcto. Es una ambigüedad del lenguaje, no del arnés.

## Detalles de zsh que_costaron tiempo

Ninguno es un problema de Mire, pero hacen el arnés silenciosamente incorrecto,
as que quedan anotados.

- **`local` dentro del cuerpo de un `for` hace que la asignación siguiente se
  imprima en vez de guardarse.** `time_ms` recogía el `date` entero como
  resultado y todos los tiempos salían mal. Todas las declaraciones están
  fuera de los bucles.
- **zsh no parte palabras de una expansión sin comillas**, así que
  `rustc $RUST_OPT` con `RUST_OPT="-C opt-level=3"` pasa un único argumento.
  Es un array: `RUST_OPT=(-C opt-level=3)`.
