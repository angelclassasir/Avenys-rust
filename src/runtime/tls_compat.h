#ifndef MIRE_TLS_COMPAT_H
#define MIRE_TLS_COMPAT_H

// ── Thread-local storage portability ──────────────────────────────────
//
// The runtime and the PAL keep per-thread state: last error, sandbox verdict,
// captured cwd, proc exit status, shell buffers. C11 spells that
// `_Thread_local`; the GNU extension spells it `__thread`. Support differs
// sharply across the C compilers Avenys has to build with:
//
//   clang / gcc                        : both spellings work
//   tcc (mob, git)                     : both spellings work
//   tcc (Arch 0.9.28rc, /usr/bin/tcc)  : neither works. `__thread` is not a
//                                        keyword there, so `static __thread int
//                                        x;` fails with "';' expected (got
//                                        'int')", and `_Thread_local` reports
//                                        "_Thread_local is not implemented".
//
// Both tcc builds report the same __TINYC__ and the same C99 __STDC_VERSION__,
// so no preprocessor test can tell them apart. The build therefore probes the
// compiler once with a real snippet (see `tcc_supports_tls` in build_support.rs)
// and defines MIRE_HAVE_TLS to 1 or 0. When it is 0 the declarations below
// degrade to ordinary static storage, so a compiler without TLS still builds
// the runtime instead of failing on six files.
//
// The consequence is documented rather than hidden: with MIRE_HAVE_TLS=0 that
// state is shared between threads instead of private to each.

#ifndef MIRE_HAVE_TLS
// Not defined means the probe did not run: assume a compiler that has TLS,
// which is true for every toolchain other than an old tcc.
#define MIRE_HAVE_TLS 1
#endif

#if MIRE_HAVE_TLS
#define MIRE_TLS __thread
// `tls_model` only affects the ELF dynamic model for shared objects. It is
// kept on the TLS path because clang and the mob tcc both accept it.
#define MIRE_TLS_MODEL __attribute__((tls_model("global-dynamic")))
#else
#define MIRE_TLS
#define MIRE_TLS_MODEL
#endif

#endif // MIRE_TLS_COMPAT_H
