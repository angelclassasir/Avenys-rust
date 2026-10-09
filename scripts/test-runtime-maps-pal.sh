#!/bin/sh
set -eu

repository_directory=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
temporary_directory=$(mktemp -d "${TMPDIR:-/tmp}/avenys-runtime-maps-pal.XXXXXX")
trap 'rm -rf "$temporary_directory"' EXIT HUP INT TERM

${CC:-cc} -std=c11 -Wall -Wextra \
    -ffunction-sections -fdata-sections \
    -I"$repository_directory/src/runtime" \
    -I"$repository_directory/src/pal" \
    "$repository_directory/tests/runtime_maps_pal_smoke.c" \
    "$repository_directory/src/runtime/collections/map/maps.c" \
    "$repository_directory/src/runtime/collections/map/maps_internal.c" \
    -Wl,--gc-sections \
    -o "$temporary_directory/runtime-maps-pal-smoke"

"$temporary_directory/runtime-maps-pal-smoke"
