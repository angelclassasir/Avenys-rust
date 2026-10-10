#!/bin/zsh
# Compare Avenys/Mire against Rust on the same calculation.
#
#   ./run.sh verify                 compare every kernel's output
#   ./run.sh measure                measure all kernels with an opacity barrier
#   ./run.sh measure <kernel>       measure one kernel
#   ./run.sh curve [kernel]         sweep optimization levels
#   ./run.sh startup                measure trivial-program startup
#
# Why there are two modes and why their seeds differ
# -------------------------------------------------------
# Both kernels produce exactly the same output, comparing both languages on
# the same calculation.
#
# In `measure`, the seed is replaced with the address of a heap allocation.
# It does not need to change between runs (ASLR may be disabled); the compiler
# simply must not know it at compile time. Without this barrier LLVM can fold
# `acc = acc * 3 + 1` into a closed form and report an impossible runtime.
#
# Why there is a noise floor
# ---------------------------
# Mire and Rust startup each cost about 1 ms. Kernels below about 25 ms measure
# startup more than computation, so the script marks them as noise.

set -u

BENCH_DIR="${0:A:h}"
KERNELS="$BENCH_DIR/kernels"
WORK="${MIRE_BENCH_WORK:-/tmp/mire-bench}"
DEFAULT_MIRE="$BENCH_DIR/../target/release/mire"
[[ -x "$DEFAULT_MIRE" ]] || DEFAULT_MIRE="$HOME/.local/bin/mire"
MIRE="${MIRE_BIN:-$DEFAULT_MIRE}"
MIRE_LIB_DIR="${MIRE_LIB_DIR:-$HOME/.owl/libs}"
MSSEC=5
# Store options in an array: zsh does not split an unquoted scalar into words,
# so a string would pass `-C opt-level=3` as one argument.
RUST_OPT=(-C opt-level=3)
# The C leg is the reference: same computation, C library only, no language
# runtime. A separate compiler binary is used so a regression in one toolchain
# cannot silently move all three legs together.
C_COMPILER=${C_COMPILER:-cc}
CFLAGS=(-O3 -march=native -std=c11)

# Noise floor: results below this value are not meaningful comparisons.
NOISE_FLOOR_MS=25

mkdir -p "$WORK"/{bin,logs}

kernels() {
  ls "$KERNELS"/*.mire 2>/dev/null | sed 's|.*/||; s|\.mire$||'
}

difficulty() {
  case "$1" in
    L1_while|P1_metodo) echo "trivial" ;;
    L2_for_lista|V2_intercambio) echo "easy" ;;
    L3_anidados|L4_break_continue|V1_muchas_locales|P2_trait_dos|C3_mixed_ops) echo "medium" ;;
    P3_cadena_impl|C1_recursion|C2_allocacion) echo "hard" ;;
    *) echo "-" ;;
  esac
}

# Compila un kernel en Rust y en Mire.
#
# mode=verify  -> semilla determinista, la salida debe coincidir
# mode=measure -> semilla opaca, el compilador no puede plegar el bucle
# optlevel     -> solo se usa en measure/curve
build_one() {
  local k="$1" mode="$2" opt="${3:-3}"

  sed 's|OPACUE(|std::hint::black_box(|g' "$KERNELS/$k.rs" > "$WORK/logs/rs_$k.rs"
  rustc $RUST_OPT -o "$WORK/bin/rust_$k" "$WORK/logs/rs_$k.rs" 2> "$WORK/logs/rust_$k.log" || {
    print -r -- "  $k: rustc failed"; head -5 "$WORK/logs/rust_$k.log"; return 1
  }

  # The C leg. Verify mode uses the deterministic seed so all three legs can be
  # compared; measure mode uses the opaque one, derived in the preamble from a
  # fresh allocation's address.
  if [[ -f "$KERNELS/$k.c" ]]; then
    cp "$KERNELS/_common.h" "$KERNELS/$k.c" "$WORK/logs/"
    # Verify needs a fixed seed on every leg; measure needs an opaque one.
    local seed_flag=()
    [[ "$mode" == "verify" ]] && seed_flag=(-DMIRE_BENCH_VERIFY)
    $C_COMPILER "${CFLAGS[@]}" "${seed_flag[@]}" -I"$WORK/logs" -o "$WORK/bin/c_$k" "$WORK/logs/$k.c" \
        2> "$WORK/logs/c_$k.log" || {
      print -r -- "  $k: cc failed"; head -5 "$WORK/logs/c_$k.log"; return 1
    }
  fi

  local d="$WORK/$k.$mode.$opt"
  mkdir -p "$d/code"
  printf '[security]\nexterns = []\n' > "$d/cfg.toml"

  # Choose the seed in one place. Measurement replaces the opaque seed
  # declaration with a pointer-derived value.
  python3 - "$KERNELS/$k.mire" "$d/code/main.mire" "$mode" <<'PY'
import sys
src, dst, mode = sys.argv[1], sys.argv[2], sys.argv[3]
text = open(src).read()
if mode == "measure":
    text = text.replace(
        'set seed = SEED_OPACA mut',
        'set sem = "x"\n    set seed = (sem :ptr) :i64 mut')
    text = text.replace('(SEED_OPACA) :f64', '(1 :i64) :f64')
    text = text.replace('SEED_OPACA', '1 :i64')
else:
    text = text.replace('SEED_OPACA', '987654321 :i64')
open(dst, 'w').write(text)
PY

  ( cd "$d" && timeout 900 "$MIRE" build code/main.mire --config cfg.toml --release \
      -O "$opt" --lib-dir "$MIRE_LIB_DIR" ) > "$WORK/logs/mire_$k.log" 2>&1
  if [[ ! -x "$d/code/bin/release/main" ]]; then
    print -r -- "  $k: Mire failed -> $(grep -oE '╰─ .*|llc: error: .*' "$WORK/logs/mire_$k.log" | head -1 | cut -c1-60)"
    return 1
  fi
  return 0
}

# Report the best of N samples in milliseconds. A scheduling outlier can only
# make the average worse, so use the best sample.
#
# Declare variables outside loops: in zsh, `local` inside a `for` body can
# make the next assignment print instead of storing its value, causing the
# complete `date` output to be mistaken for a timing result.
time_ms() {
  local best=99999999 r ms s e
  for r in $(seq 1 $MSSEC); do
    s=$(date +%s%N)
    "$@" >/dev/null 2>&1
    e=$(date +%s%N)
    ms=$(( (e - s) / 1000000 ))
    (( ms < best )) && best=$ms
  done
  print -r -- "$best"
}

cmd_verify() {
  print -r -- "Checking that Rust and Mire perform the same calculation..."
  local k fail=0 d mo ro
  for k in $(kernels); do
    [[ "$k" == "noop" ]] && continue
    build_one "$k" verify || { fail=1; continue; }
    d="$WORK/$k.verify.3"
    mo=$("$d/code/bin/release/main" 2>&1 | tail -1)
    ro=$("$WORK/bin/rust_$k" 2>&1 | tail -1)
    co=""
    [[ -x "$WORK/bin/c_$k" ]] && co=$("$WORK/bin/c_$k" 2>&1 | tail -1)
    if [[ "$mo" == "$ro" && ( -z "$co" || "$mo" == "$co" ) ]]; then
      printf "  %-19s %-9s %s\n" "$k" "match" "$mo"
    else
      printf "  %-19s %-9s rust=%s mire=%s c=%s\n" "$k" "DIFFERS" "$ro" "$mo" "$co"
      fail=1
    fi
  done
  if (( fail )); then
    print -r -- "\nOutput differs. No timings are reported until all kernels match."
    return 1
  fi
  print -r -- "\nAll kernels match. Measurements are allowed."
}

cmd_measure() {
  local only="${1:-}"
  print -r -- "Measuring (best of $MSSEC, Rust ${RUST_OPT[*]}, C $C_COMPILER -O3, Mire -O3)"
  print -r -- ""
  printf "  %-19s %-9s %10s %10s %10s %10s %10s\n" KERNEL WORKLOAD "Rust -O3" "C -O3" "Mire -O3" "Mire/C" "Mire/Rust"
  printf "  %-19s %-9s %10s %10s %10s %10s %10s\n" "-------------------" "---------" "----------" "----------" "----------" "--------" "---------"
  local k d r m c mr mc ratio_c ratio_r flag
  for k in $(kernels); do
    [[ "$k" == "noop" ]] && continue
    [[ -n "$only" && "$k" != "$only" ]] && continue
    build_one "$k" measure || continue
    d="$WORK/$k.measure.3"
    r=$(time_ms "$WORK/bin/rust_$k")
    m=$(time_ms "$d/code/bin/release/main")
    c="-"
    [[ -x "$WORK/bin/c_$k" ]] && c=$(time_ms "$WORK/bin/c_$k")
    if [[ "$c" == "-" ]]; then
      ratio_c="-"
    else
      ratio_c=$(awk -v m="$m" -v c="$c" 'BEGIN{if(c>0)printf "%.2fx", m/c; else print "-"}')
    fi
    ratio_r=$(awk -v m="$m" -v r="$r" 'BEGIN{if(r>0)printf "%.2fx", m/r; else print "-"}')
    flag=""
    (( r < NOISE_FLOOR_MS || m < NOISE_FLOOR_MS )) && flag=" <-- noise"
    printf "  %-19s %-9s %8s ms %8s ms %8s ms %9s %8s%s\n" "$k" "$(difficulty "$k")" "$r" "$c" "$m" "$ratio_c" "$ratio_r" "$flag"
  done
  print -r -- ""
  print -r -- "  (noise = below ${NOISE_FLOOR_MS} ms; startup dominates)"
}

cmd_curve() {
  local k="${1:-C2_allocacion}"
  build_one "$k" measure || return 1
  local d="$WORK/$k.measure.3"
  print -r -- "Optimization-level curve for kernel $k"
  print -r -- ""
  printf "  %-8s %12s %12s %10s\n" LEVEL Rust Mire "Mire/Rust"
  local O r m ratio
  for O in 0 1 2 3 s z; do
    sed 's|OPACUE(|std::hint::black_box(|g' "$KERNELS/$k.rs" > "$WORK/logs/rs_$k.rs"
    if rustc -C opt-level=$O -o "$WORK/bin/rust_${k}_$O" "$WORK/logs/rs_$k.rs" 2>/dev/null; then
      r=$(time_ms "$WORK/bin/rust_${k}_$O")
      rm -rf "$d/code/bin"
      ( cd "$d" && timeout 900 "$MIRE" build code/main.mire --config cfg.toml --release \
          -O "$O" --lib-dir "$MIRE_LIB_DIR" ) >/dev/null 2>&1
      m="FALLO"
      if [[ -x "$d/code/bin/release/main" ]]; then
        m="$(time_ms "$d/code/bin/release/main") ms"
      fi
      ratio="-"
      [[ "$m" != "FALLO" ]] && ratio=$(awk -v m="$m" -v r="$r" 'BEGIN{printf "%.1fx", m/r}')
      printf "  %-8s %9s ms %12s %10s\n" "-O$O" "$r" "$m" "$ratio"
    fi
  done
}

cmd_startup() {
  build_one noop measure || return 1
  local d="$WORK/noop.measure.3"
  local r m
  r=$(time_ms "$WORK/bin/rust_noop")
  m=$(time_ms "$d/code/bin/release/main")
  print -r -- "Startup time for a trivial program (best of $MSSEC):"
  print -r -- "  Rust -O3: $r ms"
  print -r -- "  Mire -O3: $m ms"
  print -r -- ""
  print -r -- "  This is the noise floor for all measurements: below ~${NOISE_FLOOR_MS} ms"
  print -r -- "  a performance comparison is not meaningful."
}

case "${1:-}" in
  verify)   cmd_verify ;;
  measure)  cmd_measure "${2:-}" ;;
  curve)    cmd_curve "${2:-}" ;;
  startup)  cmd_startup ;;
  *) print -r -- "usage: $0 {verify|measure [kernel]|curve [kernel]|startup}"; exit 2 ;;
esac
