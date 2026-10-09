#!/bin/zsh
# Comparacion Avenys/Mire contra Rust sobre el mismo calculo.
#
#   ./run.sh verify                 cada kernel: comprueba salida identica
#   ./run.sh measure                cada kernel: mide, con barrera de opacidad
#   ./run.sh measure <kernel>       solo uno
#   ./run.sh curve [kernel]         curva de nivel de optimizacion
#   ./run.sh startup                coste de arranque de un programa trivial
#
# Por que dos modos y por que la semilla cambia entre ellos
# -------------------------------------------------------
# Los dos kernel dan exactamente la misma salida, que es lo que permite
# afirmar que los dos lenguajes hacen el mismo calculo.
#
# En `measure` la semilla se sustituye por la direccion de una reserva de
# heap. No hace falta que cambie entre corridas --ASLR puede estar
# desactivado--: lo que importa es que el compilador no puede conocerla en
# tiempo de compilacion, y por tanto no puede resolver el bucle en forma
# cerrada. Sin esa barrera LLVM pliega sumas como `acc = acc * 3 + 1` en una
# Closed form y "mide" 1 ms por 50M de iteraciones, que es imposible.
#
# Por que hay un piso de ruido
# ---------------------------
# El arranque de un binario de Mire cuesta ~1 ms, igual que el de Rust. Cualquier
# kernel que termine por debajo de ~25 ms esta midiendo el arranque, no el
# calculo. El script marca esos resultados con `<-- ruido` en vez de
# presentarlos como una comparacion.

set -u

BENCH_DIR="${0:A:h}"
KERNELS="$BENCH_DIR/kernels"
WORK="${MIRE_BENCH_WORK:-/tmp/mire-bench}"
DEFAULT_MIRE="$BENCH_DIR/../target/release/mire"
[[ -x "$DEFAULT_MIRE" ]] || DEFAULT_MIRE="$HOME/.local/bin/mire"
MIRE="${MIRE_BIN:-$DEFAULT_MIRE}"
MIRE_LIB_DIR="${MIRE_LIB_DIR:-$HOME/.owl/libs}"
MSSEC=5
# Array, no string: zsh no parte palabras de una expansion sin comillas, y
# `rustc $RUST_OPT` pasaria "-C opt-level=3" como un solo argumento.
RUST_OPT=(-C opt-level=3)

# Piso de ruido: por debajo, el resultado no significa nada.
NOISE_FLOOR_MS=25

mkdir -p "$WORK"/{bin,logs}

kernels() {
  ls "$KERNELS"/*.mire 2>/dev/null | sed 's|.*/||; s|\.mire$||'
}

difficulty() {
  case "$1" in
    L1_while|P1_metodo) echo "trivial" ;;
    L2_for_lista|V2_intercambio) echo "facil" ;;
    L3_anidados|L4_break_continue|V1_muchas_locales|P2_trait_dos|C3_mixed_ops) echo "media" ;;
    P3_cadena_impl|C1_recursion|C2_allocacion) echo "dificil" ;;
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
    print -r -- "  $k: rustc fallo"; head -5 "$WORK/logs/rust_$k.log"; return 1
  }

  local d="$WORK/$k.$mode.$opt"
  mkdir -p "$d/code"
  printf '[security]\nexterns = []\n' > "$d/cfg.toml"

  # La semilla se decide aqui y en un solo sitio. En measure se reemplaza la
  # declaracion `set seed = SEED_OPACA mut` por una lectura de puntero.
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
    print -r -- "  $k: mire fallo -> $(grep -oE '╰─ .*|llc: error: .*' "$WORK/logs/mire_$k.log" | head -1 | cut -c1-60)"
    return 1
  fi
  return 0
}

# Tiempo: mejor de N, en milisegundos. El mejor y no la media porque una
# muestra contaminada por la planificacion solo puede hacer la media peor.
#
# Las variables se declaran aqui y no dentro del bucle: en zsh, un `local`
# en el cuerpo de un `for` hace que la asignacion siguiente se imprima en vez
# de guardarse, y el `date` entero se cuela en el resultado.
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
  print -r -- "Comprobando que Rust y Mire hacen el mismo calculo..."
  local k fail=0 d mo ro
  for k in $(kernels); do
    [[ "$k" == "noop" ]] && continue
    build_one "$k" verify || { fail=1; continue; }
    d="$WORK/$k.verify.3"
    mo=$("$d/code/bin/release/main" 2>&1 | tail -1)
    ro=$("$WORK/bin/rust_$k" 2>&1 | tail -1)
    if [[ "$mo" == "$ro" ]]; then
      printf "  %-19s %-9s %s\n" "$k" "identico" "$mo"
    else
      printf "  %-19s %-9s rust=%s mire=%s\n" "$k" "DIFIERE" "$ro" "$mo"
      fail=1
    fi
  done
  if (( fail )); then
    print -r -- "\nHay diferencias. No mide nada hasta que coincidan."
    return 1
  fi
  print -r -- "\nTodos los kernels coinciden. Se puede medir."
}

cmd_measure() {
  local only="${1:-}"
  print -r -- "Mediendo (mejor de $MSSEC, opt Rust ${RUST_OPT[*]}, opt Mire 3)"
  print -r -- ""
  printf "  %-19s %-9s %11s %11s %10s\n" KERNEL DIFICULTAD "Rust -O3" "Mire -O3" "Mire/Rust"
  printf "  %-19s %-9s %11s %11s %10s\n" "-------------------" "---------" "---------" "---------" "---------"
  local k d r m ratio flag
  for k in $(kernels); do
    [[ "$k" == "noop" ]] && continue
    [[ -n "$only" && "$k" != "$only" ]] && continue
    build_one "$k" measure || continue
    d="$WORK/$k.measure.3"
    r=$(time_ms "$WORK/bin/rust_$k")
    m=$(time_ms "$d/code/bin/release/main")
    ratio=$(awk -v m="$m" -v r="$r" 'BEGIN{if(r>0)printf "%.2fx", m/r; else print "-"}')
    flag=""
    (( r < NOISE_FLOOR_MS || m < NOISE_FLOOR_MS )) && flag=" <-- ruido"
    printf "  %-19s %-9s %8s ms %8s ms %9s%s\n" "$k" "$(difficulty "$k")" "$r" "$m" "$ratio" "$flag"
  done
  print -r -- ""
  print -r -- "  (ruido = por debajo de ${NOISE_FLOOR_MS} ms, se midio el arranque)"
}

cmd_curve() {
  local k="${1:-C2_allocacion}"
  build_one "$k" measure || return 1
  local d="$WORK/$k.measure.3"
  print -r -- "Curva de nivel de optimizacion, kernel $k"
  print -r -- ""
  printf "  %-8s %12s %12s %10s\n" NIVEL Rust Mire "Mire/Rust"
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
  print -r -- "Arranque de un programa trivial (mejor de $MSSEC):"
  print -r -- "  Rust -O3: $r ms"
  print -r -- "  Mire -O3: $m ms"
  print -r -- ""
  print -r -- "  Es el piso de ruido de toda medicion: por debajo de ~${NOISE_FLOOR_MS} ms"
  print -r -- "  una comparacion no significa nada."
}

case "${1:-}" in
  verify)   cmd_verify ;;
  measure)  cmd_measure "${2:-}" ;;
  curve)    cmd_curve "${2:-}" ;;
  startup)  cmd_startup ;;
  *) print -r -- "uso: $0 {verify|measure [kernel]|curve [kernel]|startup}"; exit 2 ;;
esac
