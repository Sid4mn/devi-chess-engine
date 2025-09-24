# Parallel Scaling of Alpha-Beta Tree Search (Rust + Rayon)
*A reproducible HPC probe for irregular tree search on heterogeneous architectures.*

**Motivation.** Irregular, branchy tree search stresses parallel runtimes via load imbalance and dynamic work distribution. My aim is a **clean, reproducible** scaling measurement—not a top chess engine. Consistent with **Amdahl's law**, small serial fractions bound speedup as cores increase. This project serves as a research probe for understanding parallel efficiency limits in tree-structured computations on modern heterogeneous processors.

**Implementation.** Perft-validated Rust engine (correct through depth 7); fixed-depth alpha-beta (d=4,7); **Root-level parallelization** via Rayon with lock-free parallel search. Benchmarks on **Apple M1 Pro (8-core: 6 performance + 2 efficiency)** with 5 warmup + 10 measurement runs per configuration. Statistical robustness via median timing and outlier detection.

**Parallel Scaling Results.**
| Threads | Speedup (d=4) | Efficiency | Speedup (d=7) | Efficiency |
|--------:|--------------:|-----------:|--------------:|-----------:|
| 1       | 1.00×         | 100.0%     | 1.00×         | 100.0%     |
| 2       | 1.75×         | 87.4%      | 1.88×         | 94.0%      |
| 4       | 3.17×         | 79.2%      | 3.65×         | 91.3%      |
| 8       | **4.77×**     | **59.6%**  | **6.28×**     | **78.5%**  |

![Speedup vs threads](https://raw.githubusercontent.com/Sid4mn/devi-chess-engine/v0.3.0/benchmarks/speedup_hires.png)

*Scaling Analysis.* At depth 4, achieved **4.77×** speedup (Amdahl's law, S≈0.10). At depth 7, reached **6.28×** speedup, demonstrating **Gustafson's law**—parallel efficiency improves as problem size scales. The exponential growth in search tree size (O(b^d)) overwhelms constant serial overhead, validating that deeper analysis favors parallelization.

**Fault Tolerance.** Added worker crash recovery using `catch_unwind` boundaries around individual threads. When workers panic, remaining threads continue and produce valid results. Measured overhead: 22% max (avg 15.6%) when active, zero when disabled. Demonstrates resilient parallel computing under component failure.

| Test Scenario | Overhead | Recovery |
|---------------|----------|----------|
| Fault at move 0 | 22.0% | ✅ |
| Fault at move 5 | 13.2% | ✅ |
| Fault at move 10 | 12.9% | ✅ |
| Fault at move 15 | 14.1% | ✅ |

**Stability Validation.** Soak test (100 iterations, 8 threads): median 1.414ms, p95 2.268ms, showing the engine doesn't degrade under sustained load.

**Heterogeneous Core Scheduling.** Extended the engine to measure impact of P-core vs E-core scheduling on M1 Pro using QoS-based thread biasing. Implemented four policies to isolate scheduling effects on parallel search performance.

| Policy | Searches/sec | Relative | Median Time | Description |
|--------|-------------|----------|-------------|-------------|
| None | 2.23 | 100% | 448ms | OS default scheduling |
| FastBias | 2.29 | 103% | 436ms | QoS bias to P-cores |
| EfficientBias | 0.18 | 8% | 5442ms | QoS bias to E-cores |
| Mixed (75% P) | 1.08 | 48% | 930ms | 75% P-core threads |

![Heterogeneous Impact](https://raw.githubusercontent.com/Sid4mn/devi-chess-engine/v0.3.0/benchmarks/heterogeneous_impact.png)

*Heterogeneity Analysis.* Performance cores are **13× faster** than efficiency cores (2.29 vs 0.18 searches/sec) for branch-heavy alpha-beta search—far exceeding the 3-5× expected from clock speeds alone (3.2 GHz vs 2.0 GHz). This suggests E-cores lack critical microarchitectural features (branch prediction, out-of-order execution depth) required for irregular tree search. 

The Mixed policy's disappointing **48% performance** (versus 75% expected from core ratios) reveals that naive heterogeneous scheduling fails catastrophically. The 27% performance gap suggests E-core threads become critical-path bottlenecks that stall the entire search, as root-split parallelization assigns equal work to cores with 13× performance differences. This motivates heterogeneity-aware orchestration that dynamically routes complex subtrees to P-cores while using E-cores for shallow evaluation, matching work complexity to core capability.

**Reproducibility.** Tagged releases with artifacts:
- **v0.2.2-parallel**: Original parallel scaling implementation  
- **v0.2.3-fault**: Fault tolerance implementation
- **v0.3.0**: Heterogeneous scheduling experiments

```bash
git clone https://github.com/Sid4mn/devi-chess-engine.git
cd devi-chess-engine && git checkout v0.3.0

# Parallel scaling benchmarks
./scripts/threads.sh        # generates speedup.csv and speedup.png

# Fault tolerance validation  
./scripts/run_fault.sh      # simulates worker failures

# Heterogeneous scheduling experiments
./scripts/heterogeneous.sh  # runs all 4 policies, generates impact graph
```

**Contributions.**
1. **Quantified parallel scaling limits** in irregular tree search (6.28× on 8 cores)
2. **Demonstrated fault-tolerant parallelism** with bounded overhead (15% average)
3. **Measured heterogeneous core impact** revealing 13× P vs E performance gap
4. **Identified scheduling inefficiencies** in heterogeneity-oblivious runtimes

**Next Steps.** 
1. **Work-stealing scheduler** with separate P/E core pools
2. **Heterogeneity-aware orchestrator** routing heavy subtrees to P-cores  
3. **Partitioned transposition tables** - hot entries on P-core caches, cold on E-cores
4. **PV-split parallelization** with core-aware work distribution (PV nodes -> P-cores)

---
*Contact: sid4mndev@gmail.com | GitHub: https://github.com/Sid4mn/devi-chess-engine*