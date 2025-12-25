#!/bin/bash

set -euo pipefail

echo "=== Two-Phase Scheduler Benchmark ==="
echo "Hardware: M1 Pro (8P + 2E cores)"
echo

DEPTH=${1:-7}

# Build if needed
if [[ ! -f ./target/release/devi ]]; then
    echo "Building release binary..."
    cargo build --release
fi

echo "=== Test 1: Starting Position (symmetric) ==="
echo "Depth: $DEPTH"
echo

echo "--- Baseline: P-cores only (8 threads, FastBias) ---"
./target/release/devi --depth $DEPTH --threads 8 --core-policy fast

echo
echo "--- Mixed Policy (8 threads) ---"
./target/release/devi --depth $DEPTH --threads 8 --core-policy mixed

echo
echo "--- Two-Phase Scheduler (8P + 2E) ---"
./target/release/devi --depth $DEPTH --two-phase --p-cores 8 --e-cores 2 --probe-depth 1

echo
echo "=== Test 2: Probe Depth 2 (deeper classification) ==="
./target/release/devi --depth $DEPTH --two-phase --p-cores 8 --e-cores 2 --probe-depth 2

echo
echo "=== Benchmark Complete ==="