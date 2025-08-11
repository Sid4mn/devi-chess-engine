echo "Building release binary.."
cargo build --release
echo "Running benchmarks.."
cargo run --release -- --benchmark
echo "Generating plots.."
cd scripts && python generate_speedup_plot.py
echo "Results available in benchmarks."