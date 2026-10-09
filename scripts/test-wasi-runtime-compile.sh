#!/bin/sh
set -eu

repository_directory=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
wasi_sdk_path=${WASI_SDK_PATH:-}

if [ -z "$wasi_sdk_path" ] || [ ! -x "$wasi_sdk_path/bin/clang" ]; then
    printf 'WASI_SDK_PATH must point to an installed WASI SDK.\n' >&2
    exit 2
fi

temporary_directory=$(mktemp -d "${TMPDIR:-/tmp}/avenys-wasi-runtime.XXXXXX")
trap 'rm -rf "$temporary_directory"' EXIT HUP INT TERM

compile_wasi_source() {
    source_file=$1
    object_name=$2
    "$wasi_sdk_path/bin/clang" \
        --target=wasm32-wasip1 \
        --sysroot="$wasi_sdk_path/share/wasi-sysroot" \
        -std=c11 -Wall -Wextra \
        -I"$repository_directory/src/pal" \
        -I"$repository_directory/src/pal/core" \
        -I"$repository_directory/src/runtime" \
        -c "$source_file" -o "$temporary_directory/$object_name"
}

compile_wasi_source "$repository_directory/src/pal/wasi/pal_wasi.c" pal_wasi.o
compile_wasi_source "$repository_directory/src/runtime/std/input/input.c" runtime_input.o
compile_wasi_source "$repository_directory/src/runtime/std/output/output.c" runtime_output.o
compile_wasi_source "$repository_directory/src/runtime/std/error/error.c" runtime_error.o
compile_wasi_source "$repository_directory/src/runtime/memory/safety/safety_minimal.c" runtime_safety.o
compile_wasi_source "$repository_directory/src/runtime/core/time.c" runtime_time.o

printf 'WASI PAL, standard-stream, safety, and monotonic-time modules compiled successfully.\n'
