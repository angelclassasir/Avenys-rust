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
  installer packages changed. At that checkpoint, `rt_read_tty` and direct
  process calls remained open host-boundary work; see the later process slice.
- Runtime terminal-input B2 slice (`c93ad25`): `rt_read_tty` now
  calls `pal_tty_read_line`; Linux reads the controlling terminal, while WASI
  marks the capability unsupported. The historical non-interactive `"y"`
  fallback is preserved. A `forkpty` smoke test checks real line input and a
  `setsid` process checks fallback behavior; CI uses existing system libraries.
  No installer dependency was added.
- Runtime process B2 slice (`ebdcade`): argv-based runtime capture now
  creates processes, captures stdout, reads output, and obtains exit status
  through PAL process/channel APIs instead of calling `fork`, `execvp`, `pipe`,
  `read`, and `waitpid` from runtime code. PAL process creation accepts an
  optional sandbox-authorized working directory and explicit stdout/stderr
  capture/merge flags; Linux implements these options and conventional
  `128 + signal` status reporting. Its focused smoke test covers output,
  working-directory success and failure, merged stderr, exit code 7, and
  SIGTERM. Validation passed: process, runtime I/O, files, and TTY smoke tests;
  WASI source compilation; `cargo test --locked --release -j 2` (241 library,
  14 CLI, and doc-tests); and all 12 benchmark output comparisons. C smoke
  scripts still report the previously observed comment and sandbox-path
  truncation warnings. This is one Linux process-helper slice, not completion
  of B2 or cross-platform process support. No installer dependency changed.
- Runtime vector-allocation B2 slice (`ab68030`): list creation, growth,
  concatenation, slicing, and destruction now use `pal_alloc`, `pal_realloc`,
  and `pal_free`. Size calculations reject values that cannot be represented
  by the PAL's signed size parameter. Minimal runtime selection now includes
  the PAL backend when `vecs.c` is selected, and the WASI compile smoke covers
  this module. Verification passed: Rust release tests (241 library + 14 CLI
  + doc-tests), runtime I/O/files/TTY/process smokes, WASI compilation, and
  12/12 benchmark output equivalence. A same-session C2 comparison measured
  committed allocator baseline 1,361 ms vs PAL wrapper 1,381 ms, Rust 26 ms in
  both runs. The 20 ms delta is below the harness's 25 ms noise threshold, so
  this indicates neither a measurable gain nor a confirmed regression. The
  expanded WASI check surfaces 11 existing unused-variable/parameter warnings
  from `vecs.c`; compilation succeeds. This does not finish B2 or demonstrate
  a performance improvement, and no package dependency changed.
- Runtime input-buffer B2 slice (`d447bdd`): `rt_io_read_line`,
  `rt_io_read_bytes`, and `rt_io_read_all` now allocate and release temporary
  buffers through PAL memory functions. The shared growth helper checks the
  signed PAL size limit and avoids capacity arithmetic overflow. The runtime
  I/O smoke now supplies a 257-character line, exercising buffer growth beyond
  the initial capacity. The smoke passed; the WASI compile, Rust release suite
  (241 library + 14 CLI + doc-tests), and 12/12 benchmark equivalence also
  passed. No installer dependency changed. This only migrates input buffers;
  other runtime allocations remain.
- Runtime math-allocation B2 slice (`9e1226a`): the median helper's
  temporary sorted copy now uses `pal_alloc`/`pal_free` and rejects element
  counts that overflow the PAL's signed byte-size argument. Minimal-runtime
  source selection marks `math.c` as PAL-dependent, and the WASI compile smoke
  now includes both math and vector modules. A focused C regression exercises
  odd/even median values and verifies exactly two PAL allocations and releases.
  It passes, as do the 241 library + 14 CLI Rust tests, WASI compilation, and
  all 12 benchmark output comparisons. The WASI build still surfaces the 11
  existing warnings from vector code. No installer dependency changed.
- Runtime Maybe/Result allocation B2 slice (`ab56816`): heap descriptors
  for `Maybe` and `Result` now use PAL allocation/free functions. The change
  deliberately leaves payload ownership, representation, and destructor
  behavior untouched. Minimal-runtime source selection marks `mire_types.c`
  PAL-dependent; the WASI compile smoke includes it. A focused C test verifies
  Some/Ok/Err behavior plus three matched PAL allocations/releases. It passes,
  as do the Rust release suite (241 library + 14 CLI + doc-tests), runtime I/O
  smoke, WASI compilation, and 12/12 benchmark equivalence. No installer
  dependency changed.
- Minimal managed-string B2 slice (`f149daa`): raw string copies,
  formatting buffers, owned-string buffers, registry growth, and managed frees
  now use PAL allocation/reallocation/release calls. Registry-growth failure
  is reported to `rt_managed_alloc`, which releases the unregistered buffer
  rather than silently losing ownership. Signed-size and registry-capacity
  overflow checks were added. Existing behavior is protected: values are still
  tracked in the minimal registry and static/non-owned strings are not freed.
  A C smoke checks managed copy/free, registry allocation counts, oversized
  request rejection, and preservation of a non-owned literal. The full release
  tests (241 library + 14 CLI + doc-tests), runtime I/O smoke, WASI compile, and
  12/12 benchmark equivalence pass. WASI compile now covers both full and
  minimal allocators. No installer dependency changed.
- PAL bridge scratch-buffer B2 slice (`1817898`): base64 encode/decode,
  PEM parsing, Ed25519 temporary buffers, PAL-owned shell/capture output, and
  the per-thread shell result buffer now allocate/reallocate/release through
  PAL APIs. Size arithmetic is checked before base64/PEM allocation. The
  existing file smoke now verifies `abc` encodes as `YWJj` and malformed
  base64 is rejected; process capture smoke remains green. Full Rust tests
  (241 library + 14 CLI + doc-tests), WASI compile coverage, and 12/12 benchmark
  equivalence passed. The only direct allocator-name match left in
  `helpers.c` is explanatory comment text. No installer dependency changed.
- Dictionary-container B2 slice (`e5c295b`): dictionary objects, bucket
  arrays, slot storage, and entries now use PAL allocation/reallocation/free;
  stored key/value ownership and raw-string cleanup remain unchanged. The
  focused 128-entry PAL smoke test exposed a pre-existing rehash defect: the
  rebuild used signed modulo while lookup/insertion use unsigned modulo. The
  rehash calculation now follows the same unsigned rule. Both map modules are
  selected as PAL-dependent in minimal builds and compiled by the WASI source
  gate. The focused PAL smoke passes; `cargo test --locked --release -j 2`
  passes (241 library tests, 14 CLI tests, and doc-tests); a release build
  succeeds; and the Mire map integration suite passes 3/3 with the workspace
  library path. WASI compilation and 12/12 benchmark equivalence pass. No
  benchmark timing claim is made for this slice. The first short test timeout
  was due to release LTO compilation taking about 81 seconds, not a compiler
  deadlock.
- Raw-string and formatting B2 slice (`85fbc05`): the full allocator's
  raw copy/format helpers, argv arrays, padding scratch buffers, and map
  formatting temporaries now use PAL allocation/release pairs. The runtime API
  docs state that raw helper results must be freed with `pal_free`. Minimal
  runtime source selection marks `strings.c` PAL-dependent, and the WASI gate
  compiles it. Focused C tests pass for raw-string accounting and map formatting;
  Mire string/padding tests pass 11/11, map tests pass 3/3, runtime I/O and
  process smoke tests pass, and the Rust release suite passes (241 library + 14
  CLI + doc-tests). No benchmark gain is claimed; installer dependencies are
  unchanged.
- CPU elapsed-time B2 slice (`63bee52`): `rt_cpu_elapsed_ms_str` now
  uses the existing `pal_cpu_time_ms()` capability instead of libc `clock()`;
  its input is explicitly a timestamp in the same millisecond unit. A focused
  C test verifies the result. The WASI deprecation warning for `clock()` is
  gone from `strings.c`, and the runtime audit finds no direct allocator or
  clock calls in selected runtime sources; `strings_minimal.c` remains an
  unreferenced legacy file whose integration/disposition is still open.
- Legacy minimal-string allocator B2 slice (current worktree): all raw buffer
  allocation, resize, and release in `strings_minimal.c` now goes through PAL.
  The file remains outside the selected runtime source graph, so no active
  runtime path changed. A focused smoke covers copy, concat, repeat, replace,
  padding, trim, join, argv cleanup, and balanced allocations; WASI compilation
  includes the module. The release suite passes (241 library + 14 CLI +
  doc-tests), and benchmark output equivalence passes 12/12. The file's future
  integration/disposition remains an open B1 compatibility question.
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
  operations must also move through PAL before B2 is complete. Map storage is
  routed, but map formatting temporaries and other allocator paths remain.
  B3-B6 remain
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
