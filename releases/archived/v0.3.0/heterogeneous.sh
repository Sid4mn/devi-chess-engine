#!/bin/bash
# Heterogeneous core testing for devi
# Finally got this working after fighting with macOS QoS for 2 days
# Note: This is NOT true CPU pinning, just strong hints to the scheduler

set -euo pipefail

BINARY=./target/release/devi
DEPTH=7  # depth 4 was too shallow, couldn't see the difference
THREADS=8
OUTPUT_DIR=benchmarks

echo "=== Heterogeneous Scheduling Analysis ==="
echo "========================================="
echo "Platform: Apple M1 Pro (6P+2E cores)"
echo "Configuration: Depth $DEPTH, Threads $THREADS"
echo ""

# Build if needed
if [[ ! -f "$BINARY" ]]; then
    echo "Building release binary..."
    cargo build --release
fi

mkdir -p $OUTPUT_DIR

echo "Running benchmarks (this takes ~10 mins)..."
echo ""

# Baseline - let mac decide
echo "[1/4] Testing policy: none (OS default)"
"$BINARY" --benchmark --depth $DEPTH --threads $THREADS \
    > ${OUTPUT_DIR}/policy_none.txt 2>&1

# Force onto P-cores
# QOS_CLASS_USER_INITIATED seems to work better than USER_INTERACTIVE (which lags system)
echo "[2/4] Testing policy: fast (P-cores)"
"$BINARY" --benchmark --depth $DEPTH --threads $THREADS \
    --core-policy fast \
    > ${OUTPUT_DIR}/policy_fast.txt 2>&1

# Force onto E-cores - this should be terrible
echo "[3/4] Testing policy: efficient (E-cores)"
"$BINARY" --benchmark --depth $DEPTH --threads $THREADS \
    --core-policy efficient \
    > ${OUTPUT_DIR}/policy_efficient.txt 2>&1

# Mixed - trying to match hardware ratio
# TODO: smarter work distribution instead of random, eventually a heuristic to figure out the best ratio
echo "[4/4] Testing policy: mixed (75% fast)"
"$BINARY" --benchmark --depth $DEPTH --threads $THREADS \
    --core-policy mixed --mixed-ratio 0.75 \
    > ${OUTPUT_DIR}/policy_mixed_75.txt 2>&1

echo ""
echo "Generating visualization..."

# hacky but works
cd scripts
if python compare_policies_plot.py; then
    cd ..
else
    cd ..
    echo "Plot failed - probably matplotlib"
    echo "pip install matplotlib"
    exit 1
fi

# Quick sanity check
echo ""
echo "=== Quick Results ==="
grep "Searches/second" ${OUTPUT_DIR}/policy_*.txt | sed 's/.*policy_//' | sed 's/.txt:/: /'

# Validate
if [[ -f "${OUTPUT_DIR}/heterogeneous_impact.png" ]]; then
    echo ""
    echo "Success! Check benchmarks/heterogeneous_impact.png"
    echo "open ${OUTPUT_DIR}/heterogeneous_impact.png  # macOS"
else
    echo "No graph generated"
fi

echo ""
# TODO: add CSV export
# TODO: test on Linux with real affinity