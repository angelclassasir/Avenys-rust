#!/bin/sh
set -eu

repository_directory=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
temporary_directory=$(mktemp -d "${TMPDIR:-/tmp}/avenys-runtime-cpu-time-pal.XXXXXX")
trap 'rm -rf "$temporary_directory"' EXIT HUP INT TERM

${CC:-cc} -std=c11 -Wall -Wextra \
    -ffunction-sections -fdata-sections \
    -I"$repository_directory/src/runtime" \
    -I"$repository_directory/src/pal" \
    "$repository_directory/tests/runtime_cpu_elapsed_pal_smoke.c" \
    "$repository_directory/src/runtime/core/strings.c" \
    -Wl,--gc-sections \
    -o "$temporary_directory/runtime-cpu-time-pal-smoke"

"$temporary_directory/runtime-cpu-time-pal-smoke"
