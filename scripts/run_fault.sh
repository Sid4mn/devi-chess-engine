#!/bin/bash
# Get script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "=== Fault Tolerance Testing ==="
echo "Hardware: 10 cores (8P + 2E)"
echo "Working from: $PROJECT_ROOT"
cd "$PROJECT_ROOT"

# Ensure prerequisites
if [[ ! -f "$BINARY" ]]; then
    echo "Release binary not found. Building..."
    cargo build --release
fi

# Ensure output directory exists
mkdir -p $OUTPUT_DIR

BINARY=./target/release/devi
DEPTH=7
THREADS=10  # Use all 10 cores

echo -e "\nTest 1: Baseline (no faults, 10 threads)"
$BINARY --benchmark --threads $THREADS --depth $DEPTH --warmup 3 --runs 10

echo -e "\nTest 2: Inject fault at move 0 (9/10 threads survive)"
$BINARY --benchmark --threads $THREADS --depth $DEPTH --inject-panic 0 --warmup 3 --runs 10

echo -e "\nTest 3: Inject fault at move 5 (9/10 threads survive)"
$BINARY --benchmark --threads $THREADS --depth $DEPTH --inject-panic 5 --warmup 3 --runs 10

echo -e "\nTest 4: Multiple faults (8/10 threads survive)"
$BINARY --benchmark --threads $THREADS --depth $DEPTH --inject-panic 0 --inject-panic 5 --warmup 3 --runs 10

echo -e "\nChecking crash logs..."
ls -la crashes/ 2>/dev/null || echo "No crash logs found"