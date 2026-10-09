#!/bin/sh
set -eu

repository_directory=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
temporary_directory=$(mktemp -d "${TMPDIR:-/tmp}/avenys-runtime-files.XXXXXX")
trap 'rm -rf "$temporary_directory"' EXIT HUP INT TERM

${CC:-cc} -D_GNU_SOURCE -std=c11 -Wall -Wextra \
    -ffunction-sections -fdata-sections -Wno-unused-parameter \
    -I"$repository_directory/src/pal" \
    -I"$repository_directory/src/pal/core" \
    -I"$repository_directory/src/runtime" \
    "$repository_directory/tests/runtime_file_pal_smoke.c" \
    "$repository_directory/src/runtime/pal_bridge/helpers.c" \
    "$repository_directory/src/runtime/memory/allocator/managed.c" \
    "$repository_directory/src/pal/core/pal_core.c" \
    "$repository_directory/src/pal/core/pal_dispatch.c" \
    "$repository_directory/src/pal/core/pal_sandbox.c" \
    "$repository_directory/src/pal/linux/pal_linux.c" \
    -Wl,--gc-sections -pthread -lsodium -o "$temporary_directory/runtime-file-smoke"

"$temporary_directory/runtime-file-smoke"
