# devi v0.4.0 - Comprehensive Performance Characterization

This release contains three independent performance studies on parallel chess search:

## 1. Fault Tolerance Baseline
- **Problem**: What's the cost of thread-level fault recovery without checkpointing?
- **Method**: Inject panic at move 5 after 2-ply work, measure full retry overhead
- **Result**: **100% overhead** - Rayon's all-or-nothing model discards all parallel work
- **Details**: [fault_tolerance_analysis.md](fault_tolerance_analysis.md)

## 2. Scaling Laws (Amdahl vs Gustafson)
- **Problem**: Does chess search follow Amdahl's Law (fixed problem) or Gustafson's Law (scaled problem)?  
- **Method**: Compare scaling at depth 4 vs depth 7
- **Result**: **48% serial fraction reduction** (0.27 -> 0.14) as depth increases
- **Details**: [scaling_analysis.md](scaling_analysis.md)

## 3. Heterogeneous Core Scheduling
- **Problem**: Can we exploit Apple Silicon's P/E asymmetry for chess search?
- **Method**: QoS-based thread biasing across 4 policies
- **Result**: **E-cores 12.8x slower**, mixed policy achieves only 65% of expected
- **Details**: [core_pinning_analysis.md](core_pinning_analysis.md)

## Visual Summary

### Heterogeneous Performance
![Heterogeneous Impact](heterogeneous_impact.png)

### Scaling Comparison
![Scaling](../../benchmarks/scaling_analysis/speedup_comparison.png)

## Key Findings
1. **Fault recovery needs checkpointing** - 100% overhead unacceptable for production
2. **Larger problems scale better** - Gustafson's Law dominates at higher depths
3. **E-cores unsuitable for branch-heavy work** - 12.8x gap motivates separate work queues

## Reproduce All Studies
```bash
# Clone and build
git clone https://github.com/Sid4mn/devi-chess-engine.git
cd chess-engine-rust && cargo build --release

# 1. Fault tolerance
./target/release/devi --fault-analysis --depth 7 --threads 8

# 2. Scaling laws  
./scripts/analysis/multi_depth_scaling.sh

# 3. Heterogeneous scheduling
./scripts/heterogeneous.sh
```

## Hardware
Apple M1 Pro (8P + 2E cores)

## Data Files
- Fault tolerance: [`benchmarks/fault_overhead.csv`](../../benchmarks/fault_overhead.csv)
- Heterogeneous: [`benchmarks/hetero_*.csv`](../../benchmarks/)
- Scaling: [`benchmarks/scaling_analysis/`](../../benchmarks/scaling_analysis/)

---