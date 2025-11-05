#!/bin/bash
set -euo pipefail

echo " Panic-Resilient Search Analysis"
echo "====================================="
echo "Hardware: M1 Pro (8P + 2E cores)"
echo

# Build if needed
if [[ ! -f ./target/release/devi ]]; then
    echo "Building release binary..."
    cargo build --release
fi

# Clean old logs
rm -f crashes/*.json docs/fault_analysis.json 2>/dev/null || true

echo "Running analysis (baseline + 4 fault positions)..."
echo

# Single invocation - Rust handles everything
./target/release/devi --fault-analysis --threads 10 --depth 7

# Display results if successful
if [[ -f "docs/fault_analysis.json" ]]; then
    echo
    echo "====================================="
    echo " Results Summary"
    echo "====================================="
    
    # Extract metrics (robust patterns)
    BASELINE=$(grep -o '"time_ms": [0-9.]*' docs/fault_analysis.json | head -1 | grep -o '[0-9.]*')
    
    echo "Baseline: ${BASELINE}ms (no faults)"
    echo
    echo "Fault Recovery Overhead:"
    
    # Extract overhead values (handles negative numbers)
    grep -o '"overhead_percent": -\?[0-9.]*' docs/fault_analysis.json |
        grep -o -- '-\?[0-9.]*' | 
        while read PERCENT; do
            printf "  %+.1f%%\n" "$PERCENT"
        done
fi