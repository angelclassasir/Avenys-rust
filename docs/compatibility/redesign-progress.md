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

Relevant commits on the active branches:

- Avenys: `98bdf24` (schema dependency and CI validation), `12edb4f`
  (process-spawn baseline diagnosis), `87407ee` (Linux Unix-domain sockets),
  `c48ad5f` (Mire scalar-handle adapters and minimal-runtime PAL selection).
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
- Runtime B1-B6 remains unimplemented. Preserve source files while migrating,
  route host operations through PAL, remove the `full` tier only with explicit
  compatibility handling, prove the three narrow Mire bootstrap modules, then
  measure the slab allocator against the honest C2 baseline.
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
