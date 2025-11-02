# Devi: Heterogeneous Core Analysis - Foundation for Intelligent Orchestration
*v0.3.0: A probe revealing scheduling bottlenecks in modern heterogeneous architectures*

## Summary

This release presents a comprehensive analysis of parallel scaling, fault tolerance, and heterogeneous scheduling in a chess engine probe. The key discovery-a 13× performance differential between P and E cores on Apple Silicon—significantly exceeds theoretical predictions based on clock speeds alone (1.6×), revealing fundamental architectural limitations that motivate heterogeneity-aware orchestration.

## Study Progression

### Phase 1: Parallel Scaling Baseline
![Speedup Graph](https://raw.githubusercontent.com/Sid4mn/devi-chess-engine/v0.3.0/benchmarks/speedup_hires.png)

Initial work established parallel scaling characteristics using root-split parallelization with Rayon. Achieved 6.28× speedup on 8 cores (78.5% efficiency), with Amdahl's Law analysis revealing approximately 10% serial fraction. This baseline quantified the theoretical limits of thread-level parallelism for alpha-beta search.

*Finding:* Gustafson's law validated—larger problems (depth 7) scale better than shallow (depth 4: 4.77×).

**Key insight**: Scaling efficiency degrades predictably with thread count, confirming that algorithmic improvements must complement parallelization.

### Phase 2: Fault Tolerance Engineering

Implemented panic recovery mechanisms to ensure graceful degradation under worker failure:

| Failure Point | Overhead | Recovery | Analysis |
|---------------|----------|----------|----------|
| Move 0 | 22.0% | yes  | Critical path impact |
| Move 5 | 13.2% | yes  | Typical case |
| Move 10+ | <5% | yes  | Minimal impact |
| No failures | 0% | N/A | Zero-cost abstraction |

The system maintains correctness (valid moves returned) even under worker failure, with average overhead of 15.6%-acceptable for production resilience.

### Phase 3: Heterogeneous Scheduling Discovery

![Heterogeneous Impact](https://raw.githubusercontent.com/Sid4mn/devi-chess-engine/v0.3.0/benchmarks/heterogeneous_impact.png)

This phase revealed unexpected performance characteristics of Apple Silicon's heterogeneous architecture:

```
Measured Performance (M1 Pro, Depth 7, 8 threads):
| Policy | Searches/sec | Relative | Analysis |
|--------|--------------|----------|----------|
| None (OS default) | 2.23 | 100% | Baseline |
| FastBias (P-cores) | 2.29 | 103% | Expected |
| EfficientBias (E-cores) | 0.18 | **8%** | 13× slower! |
| Mixed (75% P) | 1.08 | **48%** | Bottlenecked |
```

The 13× performance gap between P-cores and E-cores far exceeds the 1.6× clock speed ratio (3.2 GHz vs 2.0 GHz), suggesting:
- Microarchitectural differences (cache hierarchy, branch prediction)
- Lack of SMT on E-cores
- Different instruction scheduling capabilities

Most significantly, the Mixed policy achieving only 48% performance (versus 75% expected from core distribution) demonstrates critical-path bottlenecking when heterogeneous cores handle interdependent work.

## Technical Implementation

### Architecture
- **Core Engine**: Perft-validated move generation, alpha-beta search with material evaluation
- **Parallelization**: Root-split using Rayon with configurable thread pools
- **Scheduling Control**: QoS-based thread biasing via `pthread_set_qos_class_self_np`
- **Measurement Framework**: Statistical harness with warmup phases and outlier detection

### Methodology
- **Hardware Platform**: Apple M1 Pro (6 P-cores @ 3.2GHz, 2 E-cores @ 2.0GHz)
- **Benchmark Protocol**: Fixed position (startpos), depth 7 for production measurements
- **Statistical Rigor**: 5 warmup + 10 measurement iterations, median reporting
- **Reproducibility**: Automated scripts for all experiments

### Implementation Challenges Addressed
1. **QoS API Integration**: Required extensive testing to achieve reliable core biasing on macOS
2. **Thread Pool Management**: Careful isolation to prevent Rayon global pool interference  
3. **Measurement Stability**: Warmup phases essential for consistent scheduler behavior

## Analysis and Implications

### Performance Analysis

The heterogeneous scheduling results reveal three critical findings:

1. **Architectural Asymmetry**: The 13× gap indicates E-cores lack essential features for branch-heavy workloads
2. **Scheduling Inefficiency**: Naive work distribution creates critical-path dependencies
3. **Opportunity Cost**: Current parallel algorithms leave significant performance unutilized

### Study Contributions

1. **Quantified heterogeneous impact** for irregular tree search workloads
2. **Demonstrated measurement methodology** for core-tier performance analysis
3. **Identified scheduling bottlenecks** in heterogeneity-oblivious algorithms
4. **Established baseline** for heterogeneity-aware orchestration study

## Future Directions: Intelligent Orchestration

These findings directly motivate HARNESS-style orchestration approaches:

1. **Work Classification**: Shallow probing (depth 2) to estimate computational complexity
2. **Intelligent Routing**: Heavy subtrees -> P-cores, light evaluation -> E-cores
3. **Performance Targets**: Approach theoretical 75% performance for mixed configurations
4. **Metrics Framework**: Tail latency (p95/p99), work inflation, efficiency per core-tier

## Reproducibility

```bash
# Clone repository and checkout release
git clone https://github.com/Sid4mn/devi-chess-engine.git
cd devi-chess-engine && git checkout v0.3.0

# Build with native optimizations
cargo build --release

# Execute complete experimental suite
./scripts/threads.sh        # Parallel scaling analysis
./scripts/run_fault.sh      # Fault tolerance validation
./scripts/heterogeneous.sh  # Heterogeneous scheduling experiments

# Individual policy testing
./target/release/devi --benchmark --depth 7 --threads 8 --core-policy fast
./target/release/devi --benchmark --depth 7 --threads 8 --core-policy efficient
```

## Artifacts and Data

All experimental data, visualization scripts, and analysis tools are included:
- `benchmarks/`: Raw measurement data and generated plots
- `scripts/`: Automated reproduction scripts
- `releases/v0.3.0/`: This documentation and supporting materials

## Conclusion

This probe demonstrates that modern heterogeneous architectures require fundamental rethinking of parallel algorithms. The 13× performance differential and critical-path bottlenecking in mixed scheduling justify study into heterogeneity-aware orchestration systems. This work provides a foundation for intelligent work distribution strategies that match computational complexity to core capabilities.

---
*Author: Siddhant Shettiwar (sid4mndev@gmail.com)*  
*Part of ongoing study in Parallel and Distributed Systems*  
*Repository: https://github.com/Sid4mn/devi-chess-engine*