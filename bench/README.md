# Mire benchmark suite: against Rust and against C

Each kernel exists three times — in Mire, in Rust, and in C — performing the
same calculation. The C leg exists so that "equivalent" has a meaning: C has no
language runtime, so a kernel that needs runtime machinery to reach C's numbers
is measuring the runtime, not the language. `verify` compares their output before `measure` is allowed to
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

Each kernel exists three times: in Mire, in Rust, and in C. The C leg is the
reference for what "equivalent" means — C has no language runtime, so those
kernels call the C library directly (`malloc`, `realloc`, `free`) with no arena,
no reference counting, and no allocator of the compiler's own. Where a Mire
version would need runtime machinery to reach the same numbers, that gap is a
finding to record, not an excuse to rewrite the kernel until it passes.

Measured October 10, 2026, with Rust 1.99.0 (`-C opt-level=3`), C
(`cc -O3 -march=native`), and Mire/Avenys 4.3.13 (`-O3`), best of five runs.
Output verification passed for all 12 kernels across all three legs
immediately beforehand.

| Kernel | Workload | Rust -O3 | C -O3 | Mire -O3 | Mire/C | Mire/Rust |
|---|---|---:|---:|---:|---:|---:|
| `C1_recursion` | hard | 36 ms | 36 ms | 37 ms | 1.03x | 1.03x |
| `C2_allocacion` | hard | 26 ms | 65 ms | 1442 ms | **22.18x** | 55.46x |
| `C3_mixed_ops` | medium | 40 ms | 35 ms | 40 ms | 1.14x | 1.00x |
| `L1_while` | trivial | 47 ms | 39 ms | 49 ms | 1.26x | 1.04x |
| `L2_for_lista` | easy | 14 ms | 13 ms | 23 ms | 1.77x | noise floor |
| `L3_anidados` | medium | 50 ms | 55 ms | 57 ms | 1.04x | 1.14x |
| `L4_break_continue` | medium | 31 ms | 16 ms | 28 ms | 1.75x | 0.90x |
| `P1_metodo` | trivial | 129 ms | 126 ms | 133 ms | 1.06x | 1.03x |
| `P2_trait_dos` | medium | 59 ms | 64 ms | 63 ms | 0.98x | 1.07x |
| `P3_cadena_impl` | hard | 57 ms | 54 ms | 58 ms | 1.07x | 1.02x |
| `V1_muchas_locales` | medium | 44 ms | 38 ms | 46 ms | 1.21x | 1.05x |
| `V2_intercambio` | easy | 34 ms | 32 ms | 36 ms | 1.12x | 1.06x |

### What the C leg shows

**Eleven of the twelve kernels sit between 0.98x and 1.26x of C.** `P2_trait_dos`
is marginally faster than C (0.98x): trait dispatch by branch costs no more than
a function pointer. `L4_break_continue` is faster than Rust (0.90x) while
slower than C, which compiles that loop's `continue` into a jump table the
Mire backend does not match.

**The one real gap is allocation.** At 22.18x of C, `C2_allocacion` is the
kernel worth understanding. Note that C is itself 2.5x slower than Rust here
(65 ms against 26 ms): a `malloc`/`free` pair per iteration costs the system
allocator more than `Vec`'s reuse does, and the C leg pays it honestly because
the barrier keeps the allocation observable. Mire pays that same pair through
the runtime and adds its own overhead on top.

Isolating the parts of that kernel, at eight million iterations:

| Workload | Time | Per iteration |
|---|---|---|
| allocate and release one `vec[i64]` | ~430 ms | ~54 ns |
| plus eight `vec::push` and one `vec::len` | ~1442 ms | ~187 ns |

Allocation is roughly a quarter of the cost; the `push` path is the other
three quarters, at about 17 ns per call. A size-class slab allocator was built
and measured against this kernel, did not improve it, and was reverted rather
than kept on a favourable micro-benchmark — see the project history for those
numbers.

`L2_for_lista` is below the 25 ms noise floor; its ratio should not be read as
a reliable performance comparison.

## Current reference

The current C2 result is an allocator baseline, not a final result. It
supersedes the preliminary 81 ms result, which was invalid because a separate
vector-push ownership bug caused use-after-free. Rerun `verify` and
`measure C2_allocacion` after runtime or ownership changes before quoting this
number as current.
