#!/bin/bash
# VS-Rust benchmark runner

set -e

MIRE="${MIRE:-target/release/mire}"
BENCH_DIR="$(dirname "$0")"

echo "=== VS-Rust Benchmark Suite ==="
echo "Using Mire: $MIRE"

for bench in "$BENCH_DIR"/*/*.mire; do
    name=$(basename "$bench" .mire)
    echo "=== $name ==="
    if "$MIRE" build "$bench" --release -O3 -o "/tmp/vs-rust-$name" 2>&1 | tail -3; then
        "/tmp/vs-rust-$name"
        echo "exit=$?"
    else
        echo "BUILD FAILED"
    fi
    echo
done