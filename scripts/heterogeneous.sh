#!/bin/bash
set -euo pipefail

BINARY=./target/release/devi
DEPTH=7
OUT=benchmarks

[[ ! -f "$BINARY" ]] && cargo build --release
mkdir -p $OUT

# 8 threads
"$BINARY" --benchmark --depth $DEPTH --threads 8 --csv-output ${OUT}/hetero_8t_none.csv
"$BINARY" --benchmark --depth $DEPTH --threads 8 --core-policy fast --csv-output ${OUT}/hetero_8t_fast.csv
"$BINARY" --benchmark --depth $DEPTH --threads 8 --core-policy efficient --csv-output ${OUT}/hetero_8t_efficient.csv
"$BINARY" --benchmark --depth $DEPTH --threads 8 --core-policy mixed --mixed-ratio 0.8 --csv-output ${OUT}/hetero_8t_mixed.csv

# 10 threads
"$BINARY" --benchmark --depth $DEPTH --threads 10 --csv-output ${OUT}/hetero_10t_none.csv
"$BINARY" --benchmark --depth $DEPTH --threads 10 --core-policy fast --csv-output ${OUT}/hetero_10t_fast.csv
"$BINARY" --benchmark --depth $DEPTH --threads 10 --core-policy efficient --csv-output ${OUT}/hetero_10t_efficient.csv
"$BINARY" --benchmark --depth $DEPTH --threads 10 --core-policy mixed --mixed-ratio 0.8 --csv-output ${OUT}/hetero_10t_mixed.csv

python3 scripts/plot_hetero.py
echo "Plot: ${OUT}/heterogeneous_impact.png"