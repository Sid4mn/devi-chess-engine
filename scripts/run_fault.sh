#!/bin/bash
# filepath: /Users/funinc/Documents/chess-engine-rust/chess-engine-rust/scripts/run_fault.sh
set -euo pipefail

echo " Thread Recovery Analysis (v0.4.0)"
echo "Hardware: M1 Pro (8P + 2E cores)"
echo

# Build if needed
if [[ ! -f ./target/release/devi ]]; then
    echo "Building release binary..."
    cargo build --release
fi

echo "Running recovery analysis with wrapper pattern..."
echo

# Use the thread-recovery command
./target/release/devi --thread-recovery --depth 7

echo " Analysis Complete"
echo "Note: Recovery now uses generic wrapper pattern."
echo "      Works with any search strategy (baseline, parallel, hetero-phase)."