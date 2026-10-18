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
| `alloc_pressure` | Eight million short-lived vectors, each pushed eight times |
| `break_continue` | Branch-heavy loop control |
| `list_indexed_read` | Repeated indexed reads from a vector |
| `many_locals` | Live local variables and register pressure |
| `method_dispatch` | Struct method dispatch |
| `mixed_int_float` | Interleaved integer and floating-point operations |
| `nested_loops` | Two nested loops |
| `recursion_ackermann` | Recursive Ackermann workload |
| `string_impl` | Method delegation through an implementation |
| `swap_exchange` | Rotation of eight variables |
| `trait_dispatch` | Trait dispatch across implementations |
| `vec_growth` | Repeated vector growth and reallocation |
| `vec_mixed_reads` | Interleaved vector reads |
| `vec_pop_drain` | Repeated pop/drain operations |
| `while_counter` | Counter-based while loop |

`noop` is reserved for measuring process startup and is not part of the
12-kernel comparison.

## Latest measurement

Each kernel exists three times: in Mire, in Rust, and in C. The C leg is the
reference for what "equivalent" means — C has no language runtime, so those
kernels call the C library directly (`malloc`, `realloc`, `free`) with no arena,
no reference counting, and no allocator of the compiler's own. Where a Mire
version would need runtime machinery to reach the same numbers, that gap is a
finding to record, not an excuse to rewrite the kernel until it passes.

Measured October 18, 2026, with Rust 1.99.0 (`-C opt-level=3`), C
(`cc -O3 -march=native`), and the Avenys 4.4.9 release candidate (`-O3`),
best of five runs. Output verification passed for all 15 kernels across all
available legs immediately beforehand. Values are machine-specific; runs are
not pinned to a CPU, and sub-25 ms results are dominated by process startup.

| Kernel | Workload | Rust -O3 | C -O3 | Mire -O3 | Mire/C | Mire/Rust |
|---|---|---:|---:|---:|---:|---:|
| `alloc_pressure` | hard | 26 ms | 62 ms | 487 ms | 7.85x | 18.73x |
| `break_continue` | medium | 30 ms | 15 ms | 28 ms | 1.87x | 0.93x |
| `list_indexed_read` | easy | 15 ms | 14 ms | 23 ms | 1.64x | 1.53x* |
| `many_locals` | medium | 44 ms | 37 ms | 46 ms | 1.24x | 1.05x |
| `method_dispatch` | trivial | 128 ms | 125 ms | 131 ms | 1.05x | 1.02x |
| `mixed_int_float` | medium | 39 ms | 33 ms | 40 ms | 1.21x | 1.03x |
| `nested_loops` | medium | 50 ms | 54 ms | 56 ms | 1.04x | 1.12x |
| `recursion_ackermann` | hard | 36 ms | 35 ms | 37 ms | 1.06x | 1.03x |
| `string_impl` | hard | 57 ms | 53 ms | 58 ms | 1.09x | 1.02x |
| `swap_exchange` | easy | 34 ms | 32 ms | 37 ms | 1.16x | 1.09x |
| `trait_dispatch` | medium | 59 ms | 62 ms | 63 ms | 1.02x | 1.07x |
| `vec_growth` | hard | 53 ms | 75 ms | 423 ms | 5.64x | 7.98x |
| `vec_mixed_reads` | medium | 44 ms | 39 ms | 94 ms | 2.41x | 2.14x |
| `vec_pop_drain` | medium | 47 ms | 44 ms | 98 ms | 2.23x | 2.09x |
| `while_counter` | trivial | 47 ms | 38 ms | 49 ms | 1.29x | 1.04x |

### What the C leg shows

**The general compute kernels remain close to native baselines.** Ten of the
non-allocation/non-vector kernels are at or below 1.12x; the vector and
allocation kernels are the clear remaining performance work. `list_indexed_read`
is below the 25 ms noise floor and should not be used to draw a ratio conclusion.

**The recent vector change is a measured improvement, not a claim of parity.**
In paired five-run process timings, direct vector-header reads reduced
`alloc_pressure` from 791 to 553 ms (30%), `vec_growth` from 779 to 439 ms
(44%), `vec_mixed_reads` from 143 to 94 ms (34%), and `vec_pop_drain` from 163
to 102 ms (37%). These are paired before/after samples; the full-suite
best-of-five measurements above were rerun separately and show the current
cross-language comparison. Allocation and growth remain substantially slower
than Rust: `alloc_pressure` is 18.73x Rust and `vec_growth` is 7.98x.

The former 3.12x C2 result is withdrawn: it was affected by a vector-push
ownership defect and is not comparable. Always run `verify` before publishing
measurements; use paired runs when attributing a change to a code revision.
