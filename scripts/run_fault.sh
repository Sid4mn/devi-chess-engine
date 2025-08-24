#!/bin/bash
# filepath: /Users/funinc/Documents/chess-engine-rust/chess-engine-rust/scripts/run_fault.sh

echo "=== Fault Tolerance Testing ==="
echo "Building in release mode..."
cargo build --release

BINARY=../target/release/devi

echo -e "\nTest 1: Baseline (no faults)"
$BINARY --threads 4 --depth 4

echo -e "\nTest 2: Inject fault at move 0"
$BINARY --threads 4 --depth 4 --inject-panic 0

echo -e "\nTest 3: Inject fault at move 5"
$BINARY --threads 4 --depth 4 --inject-panic 5

echo -e "\nTest 4: Full fault analysis"
$BINARY --threads 4 --depth 4 --inject-panic 0 --dump-crashes

echo -e "\nChecking crash logs..."
ls -la crashes/ 2>/dev/null || echo "No crash logs found"

echo -e "\nChecking analysis results..."
cat docs/fault_analysis.json 2>/dev/null || echo "No analysis file found"