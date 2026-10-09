# Avenys, Runtime, PAL, and Owl Redesign Progress

This ledger records verified implementation milestones and known blockers. It
does not claim that the full redesign is complete.

## Active coordinated branches

- Avenys: `V4.3.13-Branch-to-verify-Avenys-Owl-protobuf-cross-decode`
- Owl: `V4.3.13-Branch-to-verify-Avenys-Owl-protobuf-cross-decode`

## Verified milestones

- Collection destruction carries semantic ownership; the C2 allocation leak
  fix is present in the branch history. Later same-target map mutation
  stabilization is recorded separately in the compatibility baseline.
- Builtins have an exhaustive registered/lowered/unsupported disposition
  test. The classification and unsupported-call diagnostic tests pass.
- The right-operand expression ascription regression (`1 + floating :i64`)
  has a type-checking regression test, which passes.
- The invalid `set value :i64 = expression` documentation form is absent from
  the tracked documentation scan. Changelog 4.3.12 records the ownership,
  builtin, type-ascription, syntax, and BETA compatibility fixes.
- CI runs the benchmark output-equivalence gate for all 12 kernels.
- The Avenys-owned protobuf schema compiles with `protoc 36.1`; CI and release
  validation install `protobuf-compiler` and run the schema check. End-user
  installers do not install `protoc`, because neither hand-written codec has
  a protobuf runtime dependency.
- Rust and Mire cross-decode the canonical `CompileReq` and `CompileResp`
  payloads in both directions. Owl CI now has a focused integration workflow.
- Linux PAL Unix-domain stream sockets have typed connect/bind operations,
  private-directory validation, mode-0600 socket nodes, inode-aware cleanup,
  and mapped missing-path errors. The two-thread exchange and path-security
  smoke test passes outside the local execution sandbox. Scalar `rt_pal_*`
  adapters preserve typed handle validation across Mire's `i64` FFI, and
  minimal-runtime source selection now includes both the bridge and PAL
  backend.
- Combined-branch verification after the bridge change: the full release suite
  (`cargo test --locked --release -j 2`) passed 237 library tests, 14 CLI
  tests, and doc-tests; `bench/run.sh verify` matched all 12 Rust/Mire kernel
  outputs. The PAL C smoke test passed with approved access outside the local
  sandbox.
- Runtime B1 move-only migration: C sources are now grouped under `core/`,
  `memory/`, `collections/`, `std/input/`, `std/output/`, `std/error/`, and
  `pal_bridge/`. The compiler recursively discovers sources, resolves
  minimal-runtime providers by their relative module paths, and release
  packaging checks that nested runtime sources are present. The former
  combined `mire_io.c` was split by responsibility without changing public
  symbols or I/O behavior. CI tests are limited to two Cargo jobs to avoid the
  previously observed linker memory thrashing. `cargo test --locked --release
  -j 2` passed 240 library tests and 14 CLI tests; `bench/run.sh verify`
  matched all 12 kernel outputs. Both `minimal` and `full` runtime tiers
  linked and ran a smoke program. A real `ireru` stdin-pipe test also exposed
  and fixed its previously missing LLVM declaration and minimal-runtime
  provider selection; the persisted `scripts/test-runtime-io.sh` smoke test
  now runs that path in CI.
- Installer and CI dependency audit for B1: no new host package is needed.
  `protobuf-compiler` remains CI-only for schema validation, not a compiler or
  runtime dependency. The README's manual package lists now match the installer's
  runtime prerequisites; the installer documents that it installs the complete
  nested source tree, and CI/release packaging asserts that the layout is
  retained.
- Runtime B2 has its first host-boundary slice: `rt_time_elapsed_ms_str` no
  longer calls `clock_gettime` directly. It uses the new
  `pal_time_monotonic_ns()` capability through `runtime/core/time.c`, keeping
  the PAL dependency isolated to that runtime unit. Linux and WASI backends
  implement the operation; the PAL socket smoke test verifies monotonic
  ordering, and the CI runtime smoke fixture calls the function from Mire in a
  minimal build. The WASI SDK compiles both the backend and runtime unit.
  B2 remains open: runtime file and process helpers still contain direct host
  calls, as do several collection and string allocation paths.
- Runtime input now goes through `pal_stdin_read`, `pal_stdin_available`, and
  `pal_stdin_is_tty`; Linux implements all three, while WASI explicitly leaves
  readiness and terminal detection unsupported. The piped-input smoke test
  checks readiness and non-terminal behavior before reading. These additions
  use existing host interfaces and add no external package dependency.
- Runtime standard output and error now use PAL writes and PAL-managed error
  flags. `dasu`, string/numeric writers, input prompts, and stream error
  queries in `runtime/std/*` no longer access host stdio directly. The Linux
  runtime smoke checks output ordering and error state; the WASI compile gate
  includes both stream modules. Panic and safety diagnostics also use the PAL;
  `scripts/test-runtime-io.sh` checks the formatted error output and preserved
  exit codes in both minimal and full runtime builds.
- CI now installs pinned WASI SDK 27.0 and compiles the WASI backend plus the
  changed standard-stream, safety, and timing modules. This development-only dependency is
  documented in the installer and README; the end-user installer does not
  install it. `scripts/test-wasi-runtime-compile.sh` is the local equivalent.
- The full managed allocator now obtains/releases its fixed 1 GiB arena through
  `pal_memory_reserve`/`pal_memory_release`. Linux preserves lazy
  `MAP_NORESERVE` behavior; WASI explicitly reports this capability unsupported
  and must use the minimal runtime tier. The allocator no longer includes or
  invokes POSIX mapping APIs, and the WASI compile gate includes this source.
  An initial substitution with generic `pal_alloc` measured C2 at 1404 ms
  (52.00x Rust), so it was rejected. The dedicated reservation API preserves
  Linux's prior mapping semantics; C2 measured 1301 ms (48.19x Rust) initially
  and 1291 ms (47.81x Rust) in a later best-of-five run against 27 ms Rust.
  All 12 benchmark outputs still match. The large gap remains and points to
  collection allocation/reallocation behavior, not a reason to claim the
  allocator is optimized.
- The managed arena's virtual-memory operations are now PAL-backed in commit
  `4950cbe`. Commit `46c6a59` routes raw-buffer, closure-environment, and
  thread-closure allocations through `pal_alloc`/`pal_free` in four runtime
  modules. Validation after this slice: 241 library tests, 14 CLI tests,
  runtime I/O/panic smoke, WASI compilation, and 12/12 benchmark output
  equivalence pass. This does not close B2: filesystem/process helpers and
  collection/string allocation paths still need an audited migration.

Relevant commits on the active branches:

- Avenys: `98bdf24` (schema dependency and CI validation), `12edb4f`
  (process-spawn baseline diagnosis), `87407ee` (Linux Unix-domain sockets),
  `c48ad5f` (Mire scalar-handle adapters and minimal-runtime PAL selection),
  `a265138` (progress ledger update), `2d75dec` (runtime source organization),
  `465c89a` (nested packaging and dependency audit), `87597da` (4.3.13
  version/changelog), and `d70ece9`/`02baedb` (standard-stream source split
  and B1 checkpoint).
- Avenys: `70ae793` routes elapsed-time formatting through a monotonic PAL
  operation. This change adds no external package dependency; CI/release checks
  verify the new runtime source is included in packaged distributions.
- Avenys: `13aaf62` routes standard input through the PAL and adds the pinned
  WASI compile check; `89b60f9` routes the runtime standard-stream APIs
  through PAL byte writes and records per-thread stream errors; `febffb3`
  verifies a clean output error state after the end-to-end input/output smoke;
  `25b0605` routes panic diagnostics through PAL and verifies the diagnostic
  format and exit status in both minimal and full runtime builds.
- Avenys: `4950cbe` moves managed-arena virtual-memory reservation behind the
  PAL; `46c6a59` routes a small set of runtime allocations through the PAL.
- Avenys: `67eefc2` routes whole-file reads, hashes, hex writes, and web-log
  appends through PAL file handles and adds the regression smoke test to CI.
- Runtime filesystem B2 slice (`67eefc2`): whole-file binary reads, file hashing,
  hex-to-file writes, and web-log append now use root-relative PAL handles.
  A C smoke test verifies exact file bytes, append behavior, hashing, and
  sandbox traversal denial. It exposed a Linux `openat2` call passing a
  creation mode without `O_CREAT`; the backend now passes mode zero for read
  opens. The smoke test is in CI and reuses existing dependencies, so no
  installer packages changed. `rt_read_tty` and direct process calls remain
  open host-boundary work.
- Runtime terminal-input B2 slice (`c93ad25`): `rt_read_tty` now
  calls `pal_tty_read_line`; Linux reads the controlling terminal, while WASI
  marks the capability unsupported. The historical non-interactive `"y"`
  fallback is preserved. A `forkpty` smoke test checks real line input and a
  `setsid` process checks fallback behavior; CI uses existing system libraries.
  No installer dependency was added.
- Owl: `ac41531` (schema dependency documentation), `0ce4193` and `9747cd4`
  (cross-decode CI and LLVM toolchain path).

## Compatibility baseline and known limits

`ecosystem-baseline-4.3.12.md` is the measured compatibility snapshot. The
MireData collection crashes were traced to same-target map mutation and fixed.
One process test remains: `proc::run::spawn("/bin/true", [])` returns `-1`;
the same failure reproduces with installed Avenys 4.3.11, so it predates the
active branch and needs focused Kioto/PAL process-handle ABI diagnosis. SDL and
Stress still encounter unresolved same-module Kioto filesystem calls. SQLite
has missing C link inputs and invalid test syntax; WASM has not yet been
verified as a WASI artifact under Wasmtime.

The local sandbox denies `bind(2)` with `EPERM`; the PAL socket regression was
therefore executed with approved elevated test access. This is an environment
restriction, not evidence that the socket implementation is unsupported.

## Remaining work

- Complete PAL C0 on all supported backends; Windows is not implemented yet.
- The planned FIFO bootstrap is not currently implementable with the exposed
  PAL: there is no FIFO-creation operation. Before using FIFOs, add a scoped,
  tested FIFO capability; otherwise finish C0 and use Unix sockets.
- Runtime B1's move-only source reorganization is implemented and verified.
  B2 has started with monotonic elapsed-time routing; its remaining host
  operations must also move through PAL before B2 is complete. B3-B6 remain
  open: remove the `full` tier
  only with explicit compatibility handling, prove the three narrow Mire
  bootstrap modules, then measure the slab allocator against the honest C2
  baseline.
- PAL C1-C6, including actual WASI/Wasmtime coverage, Windows/Wine coverage,
  removal of weak stubs, capability reporting, and the incomplete WASM package,
  remain open.
- D1-D3 and the full Owl/Avenys process protocol remain open. Cross-decoding
  payload bytes is not evidence of a working process transport or end-to-end
  build request.
- Owl's multi-language execution/build behavior (Python 3 and C source
  discovery/invocation), project lifecycle, and the final ecosystem audit and
  branch merges remain open. The minimal `owl new` scaffold is tracked in Owl.

Do not call this redesign complete until each remaining item has an explicit
verification result, ecosystem baselines are rerun, and the coordinated branch
stabilization and merges are documented.
