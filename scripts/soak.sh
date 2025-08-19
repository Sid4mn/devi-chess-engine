#!/bin/bash
# scripts/soak.sh
# Soak test automation for devi chess engine

set -euo pipefail

BINARY=./target/release/devi
THREADS=8
DEPTH=4
RUNS=100

echo "devi Chess Engine - Soak Test Automation"
echo "========================================="

# Ensure prerequisites
if [[ ! -f "$BINARY" ]]; then
    echo "Release binary not found. Building..."
    cargo build --release
fi

# Ensure output directory exists
mkdir -p docs

echo "Running soak test: $RUNS iterations at $THREADS threads, depth $DEPTH"

# Execute soak test - engine handles file writing internally
"$BINARY" --soak --threads "$THREADS" --depth "$DEPTH" --runs "$RUNS"

# Validate required output files exist
if [[ -f docs/soak_raw.txt && -f docs/soak_summary.txt ]]; then
    echo ""
    echo "Soak test completed successfully"
    echo "Raw data: docs/soak_raw.txt ($(wc -l < docs/soak_raw.txt) samples)"
    echo "Summary: docs/soak_summary.txt"
    echo ""
    echo "Summary statistics:"
    cat docs/soak_summary.txt
else
    echo "   ERROR: Expected output files not generated"
    echo "   Missing: docs/soak_raw.txt or docs/soak_summary.txt"
    echo "   Check that engine supports file output for soak tests"
    exit 1
fi