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

"$compiler" build "$repository_directory/scripts/fixtures/runtime_panic_smoke.mire" \
    --release --runtime minimal -o "$temporary_directory/runtime-panic-smoke"

set +e
"$temporary_directory/runtime-panic-smoke" \
    >"$temporary_directory/panic.stdout" 2>"$temporary_directory/panic.stderr"
panic_status=$?
set -e

if [ "$panic_status" -ne 1 ] || \
    ! grep -Fqx 'panic.mire:7:9: diagnostic smoke' "$temporary_directory/panic.stderr"; then
    printf 'Runtime panic diagnostic smoke test failed (status %s).\n' \
        "$panic_status" >&2
    sed -n '1,4p' "$temporary_directory/panic.stderr" >&2
    exit 1
fi

printf 'Runtime input, output, timing, and panic diagnostics passed through the minimal PAL runtime.\n'

"$compiler" build "$repository_directory/scripts/fixtures/runtime_panic_full_smoke.mire" \
    --release --runtime full -o "$temporary_directory/runtime-panic-full-smoke"

set +e
"$temporary_directory/runtime-panic-full-smoke" \
    >"$temporary_directory/full-panic.stdout" 2>"$temporary_directory/full-panic.stderr"
full_panic_status=$?
set -e

if [ "$full_panic_status" -ne 101 ] || \
    ! grep -Fqx 'runtime error: managed diagnostic smoke' "$temporary_directory/full-panic.stderr"; then
    printf 'Full-runtime panic diagnostic smoke test failed (status %s).\n' \
        "$full_panic_status" >&2
    sed -n '1,4p' "$temporary_directory/full-panic.stderr" >&2
    exit 1
fi

printf 'Full-runtime panic diagnostics passed through the PAL.\n'
