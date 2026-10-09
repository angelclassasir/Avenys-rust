#!/bin/sh
set -eu

repository_directory=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
temporary_directory=$(mktemp -d "${TMPDIR:-/tmp}/avenys-pal-unix-test.XXXXXX")
trap 'rm -rf "$temporary_directory"' EXIT HUP INT TERM

${CC:-cc} -D_GNU_SOURCE -std=c11 -Wall -Wextra \
    -Wno-unused-parameter -Wno-format-truncation -pthread \
    -I"$repository_directory/src/pal" \
    "$repository_directory/tests/pal_unix_socket_smoke.c" \
    "$repository_directory/src/pal/core/pal_core.c" \
    "$repository_directory/src/pal/core/pal_dispatch.c" \
    "$repository_directory/src/pal/core/pal_sandbox.c" \
    "$repository_directory/src/runtime/pal_bridge.c" \
    "$repository_directory/src/pal/linux/pal_linux.c" \
    -lsodium -o "$temporary_directory/pal-unix-smoke"

"$temporary_directory/pal-unix-smoke"
