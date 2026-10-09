# Ecosystem Compatibility Baseline — Avenys 4.3.12

Date: 2026-10-09  
Compiler: branch-built `mire` reporting Avenys 4.3.12  
Test profile: release, two jobs  
Dependency variants: `~/.owl/libs` and `~/.owl/libs-beta`

This is a measured snapshot, not a claim that every project is currently
compatible. Projects were copied into an isolated directory under `/tmp` before
tests or builds ran. Their checkout contents, tracked build caches, and dirty
work were not modified by this audit. Results that depend on a library
directory name that directory explicitly.

## Results

| Project | Dependency context | Result |
|---|---|---|
| Avenys compiler | Current branch | `cargo test --locked --release -j 2`: 233 library tests and 14 binary tests passed; doc-tests passed (none defined). `bench/run.sh verify`: all 12 Mire/Rust outputs matched. |
| `testproj` | `~/.owl/libs` | 1/1 test passed. |
| `token` | `~/.owl/libs` (active Kioto 3.0 tree) | 5/5 tests passed. |
| `token` | `~/.owl/libs-beta` | Could not resolve its declared SDL dependency; this library directory contains Kioto, Mire, and testlib, but not SDL. This is a dependency-resolution failure, not a compiler regression result. |
| MireData, initial run | `~/.owl/libs-beta` | 48 passed, 26 failed, 74 total. `advanced` and `collections` test binaries terminated with signal 11; `proc` had assertion failures. |
| MireData, after map-rebind fix | `~/.owl/libs-beta` | 77 passed, 5 failed, 82 total (including isolated probes and the new map regression). `advanced`, `collections`, and every map probe passed. The remaining failures are in `proc`; investigate separately. |
| SDL | `~/.owl/libs` | 47 tests failed before execution because generated test sources could not resolve `fs.owl.build_level`. The same diagnostic appeared across all test files. |
| Stress | `~/.owl/libs` | No test files were found. Building `code/main.mire` failed on unresolved `fs.owl.build_level` at line 509. |
| SQLite | `~/.owl/libs` | 8 generated tests failed during compilation. The linker reported missing `sqlite3_*` and `mire_sqlite_text_to_str` symbols; another test source contains an unexpected semicolon at line 13. No runtime behavior was tested. |
| WASM package | Isolated package root supplied as library directory | `code/mod.mire` built as a host shared library. This does not verify a WASI target, wasmtime execution, or the package's declared `wasm_*` externs. No test files were present. |

## Interpretation and follow-up

The results show why an ecosystem-wide pass/fail count is misleading: the
outcome changes with the Kioto library directory, and several suites fail
before their tests execute. The `fs.owl.build_level` failure is a reproducible
compiler-facing blocker in SDL and Stress, but this snapshot alone does not
prove whether it is caused by Avenys, Owl-generated configuration, or their
integration. It must be isolated with a minimal generated-source reproduction
before changing qualified-call validation.

The initial collection SIGSEGVs in MireData were reproduced with two
`map::set` calls that rebind the same variable. Lowering released the old map
even though the mutator returned the same map pointer. The Avenys follow-up fix
now exempts same-target map mutators as well as vector push; the isolated probe
passes in debug and release, and the Map suite passes 3/3. The MireData rerun
has no collection crashes; its five remaining failures are in `proc` and are
still open. SQLite needs its C link inputs and invalid test syntax separated
before it can serve as a compiler compatibility signal. The WASM host build is
only a compile smoke test; C2 must still validate the actual WASI target and
runtime.

Re-run this matrix after runtime/PAL and protocol milestones. Preserve the
dependency directory and command-line options so later results remain
comparable. Do not label a failure a regression or pre-existing until the same
case has been run against a pinned baseline revision.
