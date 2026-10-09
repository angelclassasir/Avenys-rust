#!/bin/sh
set -eu

repository_directory=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
temporary_directory=$(mktemp -d "${TMPDIR:-/tmp}/avenys-runtime-io.XXXXXX")
trap 'rm -rf "$temporary_directory"' EXIT HUP INT TERM

compiler=${MIRE_BIN:-"$repository_directory/target/release/mire"}
"$compiler" build "$repository_directory/scripts/fixtures/runtime_io_smoke.mire" \
    --release --runtime minimal -o "$temporary_directory/runtime-io-smoke"

actual_output=$(printf 'verified\n' | "$temporary_directory/runtime-io-smoke")
expected_output='Input: verified'
if [ "$actual_output" != "$expected_output" ]; then
    printf 'runtime I/O smoke test failed: expected <%s>, got <%s>\n' \
        "$expected_output" "$actual_output" >&2
    exit 1
fi

printf 'Runtime input smoke test passed through the minimal runtime.\n'
