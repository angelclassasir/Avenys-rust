# Mire versus Rust benchmark suite

Each kernel has a Mire source and a Rust source that perform the same
calculation. `verify` compares their output before `measure` is allowed to
report timings. This is a focused language/runtime comparison, not a claim
that every application will have the same performance profile.

## Commands

```sh
./run.sh verify
./run.sh measure
./run.sh measure C2_allocacion
./run.sh curve C2_allocacion
./run.sh startup
```

`verify` is a required precondition for measurements. If any kernel differs,
`measure` exits without producing benchmark numbers.

The script uses `MIRE_BIN` when set; otherwise it selects the compiler built in
this repository, falling back to `~/.local/bin/mire` only if that binary is not
available. `MIRE_LIB_DIR` selects the Mire library directory. `MIRE_BENCH_WORK`
sets the isolated work directory (default: `/tmp/mire-bench`).

## Kernels

| Kernel | Workload |
|---|---|
| `L1_while` | Counter-based while loop |
| `L2_for_lista` | Indexed reads from a dynamic list |
| `L3_anidados` | Two nested loops |
| `L4_break_continue` | `break` and `continue` control flow |
| `V1_muchas_locales` | 24 live local variables and register pressure |
| `V2_intercambio` | Rotation of eight variables |
| `P1_metodo` | Struct method dispatch |
| `P2_trait_dos` | Trait dispatch across two implementations |
| `P3_cadena_impl` | Method delegation through a trait |
| `C1_recursion` | Recursive Ackermann workload |
| `C2_allocacion` | Eight million iterations, each creating a vector |
| `C3_mixed_ops` | Interleaved integer and floating-point operations |

`noop` is reserved for measuring process startup and is not part of the
12-kernel comparison.

## Latest measurement

Measured October 9, 2026, with Rust 1.99.0 (`-C opt-level=3`) and Mire/Avenys
4.3.11 (`-O3`), best of five runs. Output verification passed for all 12
kernels immediately beforehand.

| Kernel | Workload | Rust | Mire | Mire/Rust |
|---|---|---:|---:|---:|
| `C1_recursion` | hard | 44 ms | 37 ms | 0.84x |
| `C2_allocacion` | hard | 26 ms | 1291 ms | **49.65x** |
| `C3_mixed_ops` | medium | 39 ms | 40 ms | 1.03x |
| `L1_while` | trivial | 47 ms | 49 ms | 1.04x |
| `L2_for_lista` | easy | 15 ms | 24 ms | noise floor |
| `L3_anidados` | medium | 50 ms | 56 ms | 1.12x |
| `L4_break_continue` | medium | 29 ms | 27 ms | 0.93x |
| `P1_metodo` | trivial | 127 ms | 131 ms | 1.03x |
| `P2_trait_dos` | medium | 59 ms | 62 ms | 1.05x |
| `P3_cadena_impl` | hard | 57 ms | 58 ms | 1.02x |
| `V1_muchas_locales` | medium | 44 ms | 46 ms | 1.05x |
| `V2_intercambio` | easy | 35 ms | 37 ms | 1.06x |

`L2_for_lista` is below the 25 ms noise floor and its ratio should not be
interpreted as a reliable performance comparison. Most other measured kernels
remain close to Rust; the outstanding measured gap is vector allocation.

## Measurement safeguards

- **Opaque input:** verification uses a deterministic seed. Measurement
  substitutes a heap address so the compiler cannot fold the loop into a
  closed-form expression. The Rust equivalent uses
  `std::hint::black_box`.
- **Serial dependency:** each iteration depends on the previous accumulator;
  otherwise the optimizer may collapse the loop.
- **Noise floor:** results below approximately 25 ms are marked as startup
  noise instead of being presented as a meaningful performance comparison.
- **Repeated samples:** the script reports the best of five runs to reduce
  scheduler-noise impact.
- **Isolated build products:** generated binaries and logs live below the
  configured work directory.

## Shell pitfalls already fixed

The script is written for zsh. A `local` declaration inside a `for` loop can
cause the following assignment to print instead of being stored; that once
made the complete `date` output look like a timing result. Variables are now
declared outside loop bodies. Also, zsh does not split an unquoted scalar into
command arguments, so Rust compiler options are stored in an array rather than
a space-delimited string.

## Current reference

The current C2 result is an allocator baseline, not a final result. It
supersedes the preliminary 81 ms result, which was invalid because a separate
vector-push ownership bug caused use-after-free. Rerun `verify` and
`measure C2_allocacion` after runtime or ownership changes before quoting this
number as current.
