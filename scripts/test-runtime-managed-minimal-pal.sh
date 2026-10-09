#!/bin/sh
set -eu

repository_directory=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
temporary_directory=$(mktemp -d "${TMPDIR:-/tmp}/avenys-managed-minimal-pal.XXXXXX")
trap 'rm -rf "$temporary_directory"' EXIT HUP INT TERM

${CC:-cc} -std=c11 -Wall -Wextra \
    -ffunction-sections -fdata-sections \
    -I"$repository_directory/src/runtime" \
    -I"$repository_directory/src/pal" \
    "$repository_directory/tests/runtime_managed_minimal_pal_smoke.c" \
    "$repository_directory/src/runtime/memory/allocator/managed_minimal.c" \
    -Wl,--gc-sections \
    -o "$temporary_directory/runtime-managed-minimal-pal-smoke"

"$temporary_directory/runtime-managed-minimal-pal-smoke"
