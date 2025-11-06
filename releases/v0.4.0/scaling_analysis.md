# Parallel Scaling Analysis: Amdahl vs Gustafson in Chess Engine Search

## Key Finding
**Larger problems scale better**: Increasing search depth from 4 to 7 reduces serial fraction by 48%, enabling 6.73x speedup (vs 4.56x) on 10 cores.

**Useful Info:** <br>
In alpha-beta search:
- **Serial work**: Root move generation, thread coordination, result aggregation
- **Parallel work**: Searching subtrees for each root move
- **Scaling parameter**: Search depth (exponential growth in nodes)


## Experimental Setup

- **Platform**: Apple M1 Pro
- **Cores**: 10 total (8 Performance + 2 Efficiency)
- **Memory**: Unified architecture with shared L2 cache
- **Threading**: Rayon thread pool with root-level parallelism

- **Engine**: Devi v0.4.0 (perft-validated through depth 7)
- **Parallelization**: Root-split parallel alpha-beta
- **Protocol**: 5 warmup runs + 10 measurement runs
- **Metrics**: Median time, speedup, parallel efficiency
- **Thread configurations**: 1, 2, 4, 6, 8, 10

### Test Configurations
| Parameter | Depth 4 (Amdahl) | Depth 7 (Gustafson) | Ratio |
|-----------|------------------|---------------------|-------|
| **Tree Size** | ~1.5M nodes | ~3.2B nodes | 2,133x |
| **Serial Overhead** | ~0.7ms | ~0.9ms | 1.3x |
| **Total Time (1T)** | 6.05ms | 2,715ms | 449x |
| **Serial Fraction** | 11.5% | 6.0% | 0.52x |

## Empirical Results

### Performance Data
| Metric | Depth 4 (Small) | Depth 7 (Large) | Improvement |
|--------|-----------------|-----------------|-------------|
| **Serial Fraction** | 11.5% | 6.0% | **48% reduction** |
| **Speedup @ 8 threads** | 4.80x (60% eff.) | 6.23x (78% eff.) | **+30% speedup** |
| **Speedup @ 10 threads** | 4.56x (46% eff.) | 6.73x (67% eff.) | **+48% speedup** |
| **Max Theoretical** | 8.7x | 16.7x | **92% higher ceiling** |

### Visual Analysis

![Speedup Comparison](benchmarks/scaling_analysis/speedup_comparison.png)
*Depth 7 (large problem) maintains near-linear scaling while Depth 4 (small) plateaus early*

![Efficiency Comparison](benchmarks/scaling_analysis/efficiency_comparison.png)  
*17.9% efficiency gap at 8 threads demonstrates Gustafson's principle: larger problems utilize parallelism better*

## Key Observations

1. **Small problems hit Amdahl's wall**: Depth 4 peaks at 4.80x then **plateaus/regresses**
2. **Large problems follow Gustafson**: Depth 7 continues scaling to 6.73x
3. **Serial work stays constant (~0.8ms) while parallel work grows 2000x** (1.5M -> 3.2B nodes)

## Practical Implications

- **For shallow search (depth ≤4)**: Use P-cores only (`--threads 8`)
- **For deep search (depth ≥7)**: Use all cores (`--threads 10`)  
- **Hardware insight**: E-cores need sufficient work to overcome coordination overhead

## Reproduction

```bash
# Clone and build
git clone https://github.com/Sid4mn/devi-chess-engine.git
cd devi-chess-engine && cargo build --release

# Generate all data and plots
./scripts/analysis/multi_depth_scaling.sh

# Or run individual tests
./target/release/devi --benchmark --benchmark-sweep --depth 4 --csv-output benchmarks/depth4.csv
./target/release/devi --benchmark --benchmark-sweep --depth 7 --csv-output benchmarks/depth7.csv
```

## Conclusion

This empirically validates the fundamental difference between **strong scaling** (fixed problem, diminishing returns) and **weak scaling** (scaled problem, sustained efficiency). Chess engines naturally follow Gustafson's model: deeper searches with more cores, not faster shallow searches.

---
*Hardware: Apple M1 Pro (8P+2E cores) | Engine: Devi v0.4.0*