#!/bin/bash
# scripts/threads.sh
# Threads script automation for devi chess engine

set -euo pipefail

BINARY=./target/release/devi
DEPTH=7
OUTPUT_DIR=benchmarks
CSV_FILE=benchmarks/speedup.csv
PNG_FILE=benchmarks/speedup.png

echo "devi Chess Engine - Threads Test Automation"

echo "Hardware Configuration:"
echo "  Total cores: $(sysctl -n hw.physicalcpu)"
echo "  P-cores: $(sysctl -n hw.perflevel0.physicalcpu)"
echo "  E-cores: $(sysctl -n hw.perflevel1.physicalcpu)"
echo ""

# Ensure prerequisites
if [[ ! -f "$BINARY" ]]; then
    echo "Release binary not found. Building..."
    cargo build --release
fi

# Ensure output directory exists
mkdir -p $OUTPUT_DIR

echo "Running benchmark at depth $DEPTH for threads [1, 2, 4, 6, 8, 10]..."

# Execute benchmark (this calls export_benchmark_csv internally)
"$BINARY" --benchmark --depth "$DEPTH" --threads 10

# Validate CSV was created and is non-empty
if [[ -f "$CSV_FILE" && -s "$CSV_FILE" ]]; then
    echo "Benchmark completed successfully"
    echo "Results: $CSV_FILE ($(wc -l < "$CSV_FILE") lines)"
else
    echo "ERROR: Expected CSV file not generated or empty"
    echo "Missing: $CSV_FILE"
    echo "Check that --benchmark flag works correctly"
    exit 1
fi

# Generate plots (cd into scripts/ for correct ../benchmarks/ path)
echo "Generating speedup plots..."
cd scripts
if python generate_speedup_plot.py; then
    cd ..
else
    cd ..
    echo "WARNING: Plot generation failed (check matplotlib/pandas installation)"
    echo "Benchmark data is still available in $CSV_FILE"
    exit 1
fi

# Validate PNG was created
if [[ -f "$PNG_FILE" ]]; then
    echo ""
    echo "Threads automation completed successfully"
    echo ""
    echo "Generated files:"
    echo "  Data: $CSV_FILE ($(stat -f%z "$CSV_FILE" 2>/dev/null || stat -c%s "$CSV_FILE" 2>/dev/null || echo "unknown") bytes)"
    echo "  Plot: $PNG_FILE ($(stat -f%z "$PNG_FILE" 2>/dev/null || stat -c%s "$PNG_FILE" 2>/dev/null || echo "unknown") bytes)"
    echo "  High-res: benchmarks/speedup_hires.png"
    echo ""
    echo "View results:"
    echo "  cat $CSV_FILE"
    echo "  open $PNG_FILE  # macOS"
else
    echo "WARNING: PNG file not generated"
    echo "Expected: $PNG_FILE"
    echo "Benchmark data is available but visualization failed"
    exit 1
fi