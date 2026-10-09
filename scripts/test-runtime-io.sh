#!/bin/sh
set -eu

repository_directory=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
temporary_directory=$(mktemp -d "${TMPDIR:-/tmp}/avenys-runtime-io.XXXXXX")
trap 'rm -rf "$temporary_directory"' EXIT HUP INT TERM

compiler=${MIRE_BIN:-"$repository_directory/target/release/mire"}
"$compiler" build "$repository_directory/scripts/fixtures/runtime_io_smoke.mire" \
    --release --runtime minimal -o "$temporary_directory/runtime-io-smoke"

actual_output=$(printf 'verified\n' | "$temporary_directory/runtime-io-smoke")
actual_input=$(printf '%s\n' "$actual_output" | sed -n '1p')
actual_elapsed=$(printf '%s\n' "$actual_output" | sed -n '2p')
if [ "$actual_input" != 'Input: verified' ] || \
    [ -z "$actual_elapsed" ] || \
    ! printf '%s' "$actual_elapsed" | grep -Eq '^[0-9]+$'; then
    printf 'runtime I/O/time smoke test failed: got <%s>\n' \
        "$actual_output" >&2
    exit 1
fi

printf 'Runtime input and monotonic-time smoke tests passed through the minimal runtime.\n'
