#!/bin/bash
# Two-Phase Scheduler Benchmark Suite
# Compares baseline, fast-bias, and two-phase scheduling strategies

set -e

DEPTH=${1:-7}
WARMUP=${2:-5}
RUNS=${3:-10}
OUTPUT_DIR="benchmarks/v0.5.0"

echo "=== TWO-PHASE SCHEDULER BENCHMARK SUITE ==="
echo "Depth: $DEPTH, Warmup: $WARMUP, Runs: $RUNS"
echo "Output: $OUTPUT_DIR"
echo ""

mkdir -p "$OUTPUT_DIR"

# Build in release mode
echo "Building in release mode..."
cargo build --release

BINARY="./target/release/devi"

# Run the comprehensive two-phase benchmark
echo ""
echo "Running two-phase benchmark..."
$BINARY --two-phase-benchmark \
    --depth "$DEPTH" \
    --warmup "$WARMUP" \
    --runs "$RUNS" \
    --p-cores 8 \
    --e-cores 2 \
    --csv-output "$OUTPUT_DIR/two_phase_benchmark.csv"

echo ""
echo "=== BENCHMARK COMPLETE ==="
echo "Results saved to $OUTPUT_DIR/two_phase_benchmark.csv"
echo ""
echo "To generate plots, run:"
echo "  python3 scripts/analyze_two_phase.py"
