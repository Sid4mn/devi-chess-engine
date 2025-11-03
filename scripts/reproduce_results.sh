set -euo pipefail

echo "=== Full Benchmark Suite ==="
# Record cores.
echo "Hardware: $(sysctl -n hw.physicalcpu) cores ($(sysctl -n hw.perflevel0.physicalcpu)P + $(sysctl -n hw.perflevel1.physicalcpu)E)"
echo ""

# Clean old results
rm -f benchmarks/speedup.csv

echo "Building release binary..."
cargo build --release
echo ""

echo "Running parallel scaling benchmark..."
./target/release/devi --benchmark --benchmark-sweep --depth 7 --warmup 3 --runs 10

# Verify the CSV was created
if [ ! -f benchmarks/speedup.csv ]; then
    echo "ERROR: Benchmark failed to create speedup.csv"
    exit 1
fi

echo ""
echo "Generating visualization..."
cd scripts && python generate_speedup_plot.py

echo ""
echo "Results available in benchmarks."