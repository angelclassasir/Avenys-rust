#!/bin/sh
set -eu

repository_directory=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
temporary_directory=$(mktemp -d "${TMPDIR:-/tmp}/avenys-runtime-managed-release.XXXXXX")
trap 'rm -rf "$temporary_directory"' EXIT HUP INT TERM

${CC:-cc} -std=c11 -Wall -Wextra \
    -ffunction-sections -fdata-sections \
    -I"$repository_directory/src/runtime" \
    -I"$repository_directory/src/pal" \
    "$repository_directory/tests/runtime_managed_release_smoke.c" \
    "$repository_directory/src/runtime/memory/allocator/managed.c" \
    -Wl,--gc-sections \
    -o "$temporary_directory/runtime-managed-release-smoke"

"$temporary_directory/runtime-managed-release-smoke"