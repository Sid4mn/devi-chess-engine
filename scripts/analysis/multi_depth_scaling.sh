#!/bin/bash
# scripts/multi_depth_scaling.sh
# Multi-depth scaling analysis for Amdahl vs Gustafson demonstration

set -euo pipefail

# Setup
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$PROJECT_ROOT"

BINARY=./target/release/devi
OUTPUT_DIR=benchmarks/scaling_analysis
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Configuration
THREADS=(1 2 4 6 8 10)
WARMUPS=5
RUNS=10

echo "=========================================="
echo " Multi-Depth Scaling Analysis"
echo " Demonstrating Amdahl vs Gustafson Laws"
echo "=========================================="
echo
echo "Hardware Configuration:"
echo "  Total cores: $(sysctl -n hw.physicalcpu)"
echo "  P-cores: $(sysctl -n hw.perflevel0.physicalcpu)"
echo "  E-cores: $(sysctl -n hw.perflevel1.physicalcpu)"
echo
echo "Test Configuration:"
echo "  Depths: 4 (Amdahl) and 7 (Gustafson)"
echo "  Threads: ${THREADS[@]}"
echo "  Protocol: $WARMUPS warmups, $RUNS measurement runs"
echo

# Ensure binary exists
if [[ ! -f "$BINARY" ]]; then
    echo "Building release binary..."
    cargo build --release
fi

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Clean previous CSVs from benchmarks/ (we'll generate fresh ones)
rm -f benchmarks/speedup.csv

# Function to run benchmark at specific depth
run_depth_benchmark() {
    local DEPTH=$1
    local OUTPUT_CSV=$2
    local DESCRIPTION=$3
    
    echo "================================================"
    echo " $DESCRIPTION"
    echo " Depth: $DEPTH"
    echo "================================================"
    
    # Use --csv-output flag instead of copying files
    $BINARY --benchmark --benchmark-sweep --depth $DEPTH \
            --warmup $WARMUPS --runs $RUNS \
            --csv-output "$OUTPUT_CSV"
    
    echo "Results saved to: $OUTPUT_CSV"
    echo
}


# Run Amdahl's Law demonstration (small problem, depth 4)
echo
echo "PHASE 1: Amdahl's Law (Strong Scaling)"
echo "======================================="
echo "Fixed small problem (depth 4), adding threads"
echo "Expect: Diminishing returns, serial fraction dominates"
echo

run_depth_benchmark 4 "$OUTPUT_DIR/speedup_d4.csv" "Amdahl's Law Demonstration"

# Extract key metrics for depth 4
echo "Quick Analysis (Depth 4):"
tail -n +2 "$OUTPUT_DIR/speedup_d4.csv" | while IFS=',' read -r threads policy median_ms sps speedup efficiency; do
    printf "  %2d threads: %6.2fx speedup, %5.1f%% efficiency\n" \
           "$threads" "$speedup" "$efficiency"
done
echo

# Run Gustafson's Law demonstration (large problem, depth 7)
echo "PHASE 2: Gustafson's Law (Weak Scaling)"
echo "========================================"
echo "Larger problem (depth 7), utilizing added threads"
echo "Expect: Better efficiency, parallel work dominates"
echo

run_depth_benchmark 7 "$OUTPUT_DIR/speedup_d7.csv" "Gustafson's Law Demonstration"

# Extract key metrics for depth 7
echo "Quick Analysis (Depth 7):"
tail -n +2 "$OUTPUT_DIR/speedup_d7.csv" | while IFS=',' read -r threads policy median_ms sps speedup efficiency; do
    printf "  %2d threads: %6.2fx speedup, %5.1f%% efficiency\n" \
           "$threads" "$speedup" "$efficiency"
done
echo

# Save metadata
cat > "$OUTPUT_DIR/metadata_${TIMESTAMP}.txt" <<EOF
Multi-Depth Scaling Analysis
Generated: $(date)
Hardware: $(sysctl -n hw.physicalcpu) cores ($(sysctl -n hw.perflevel0.physicalcpu)P + $(sysctl -n hw.perflevel1.physicalcpu)E)
Binary: $BINARY
Warmups: $WARMUPS
Runs: $RUNS

Files Generated:
- speedup_d4.csv: Depth 4 results (Amdahl's Law)
- speedup_d7.csv: Depth 7 results (Gustafson's Law)
EOF

echo "Generated files:"
echo "  - $OUTPUT_DIR/speedup_d4.csv (Amdahl)"
echo "  - $OUTPUT_DIR/speedup_d7.csv (Gustafson)"
echo "  - $OUTPUT_DIR/metadata_${TIMESTAMP}.txt"
echo
echo "================================================"
echo " Generating Comparison Plots"
echo "================================================"

# Navigate to scripts directory and run Python analysis
cd "$PROJECT_ROOT/scripts"
if command -v python3 &> /dev/null; then
    python3 analysis/compare_scaling.py
elif command -v python &> /dev/null; then
    python analysis/compare_scaling.py
else
    echo "ERROR: Python not found. Please run manually:"
    echo "  cd scripts && python analysis/compare_scaling.py"
    exit 1
fi

# Check if plot was created
if [[ -f "$PROJECT_ROOT/$OUTPUT_DIR/amdahl_vs_gustafson.png" ]]; then
    echo
    echo "  Analysis complete! View results:"
    echo "  open $PROJECT_ROOT/$OUTPUT_DIR/amdahl_vs_gustafson.png"
else
    echo "  Plot generation may have failed"
fi

echo
echo "================================================"
echo " FULL ANALYSIS COMPLETE"