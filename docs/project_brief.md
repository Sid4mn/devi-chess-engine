# Parallel Scaling of Alpha-Beta Tree Search (Rust + Rayon)
*A reproducible HPC probe for irregular tree search on an 8-core CPU.*

**Motivation.** Irregular, branchy tree search stresses parallel runtimes via load imbalance and dynamic work distribution. My aim is a **clean, reproducible** scaling measurement-not a top chess engine. Consistent with **Amdahl's law**, small serial fractions bound speedup as cores increase. This project serves as a research probe for understanding parallel efficiency limits in tree-structured computations.

**Implementation.** Perft-validated Rust engine (correct through depth 7); fixed-depth alpha-beta (d=4); **Root-level parallelization** via Rayon with lock-free parallel search. Benchmarks on **Apple M1 Pro (8-core: 6 performance + 2 efficiency)** with 5 warmup + 10 measurement runs per configuration. Statistical robustness via median timing and outlier detection.

**Results.**
| Threads | Speedup (d=4) | Efficiency | Speedup (d=7) | Efficiency |
|--------:|--------------:|-----------:|--------------:|-----------:|
| 1       | 1.00×         | 100.0%     | 1.00×         | 100.0%     |
| 2       | 1.75×         | 87.4%      | 1.88×         | 94.0%      |
| 4       | 3.17×         | 79.2%      | 3.65×         | 91.3%      |
| 8       | **4.77×**     | **59.6%**  | **6.28×**     | **78.5%**  |

![Speedup vs threads](https://raw.githubusercontent.com/Sid4mn/devi-chess-engine/v0.2.3-fault/benchmarks/speedup_hires.png)

*Scaling Analysis.* At depth 4, achieved **4.77×** speedup (Amdahl's law, S≈0.10). At depth 7, reached **6.28×** speedup, demonstrating **Gustafson's law** - parallel efficiency improves as problem size scales. The exponential growth in search tree size (O(b^d)) overwhelms constant serial overhead, validating that deeper analysis favors parallelization.

**Fault Tolerance.** Added worker crash recovery using `catch_unwind` boundaries around individual threads. When workers panic, remaining threads continue and produce valid results. Measured overhead: 22% max (avg 15.6%) when active, zero when disabled. Demonstrates resilient parallel computing under component failure.

| Test Scenario | Overhead | Recovery |
|---------------|----------|----------|
| Fault at move 0 | 22.0% | ✓ |
| Fault at move 5 | 13.2% | ✓ |
| Fault at move 10 | 12.9% | ✓ |
| Fault at move 15 | 14.1% | ✓ |

**Stability Validation.** Soak test (100 iterations, 8 threads): median 1.414ms, p95 2.268ms, showing the engine doesn't degrade under sustained load.

**Reproducibility.** Tagged release: **v0.2.3-fault** with artifacts.

```bash
git clone https://github.com/Sid4mn/devi-chess-engine.git
cd devi-chess-engine && git checkout v0.2.3-fault
./scripts/threads.sh   # regenerates benchmarks/speedup.csv and benchmarks/speedup.png
./scripts/run_fault.sh    # fault tolerance validation
```

**Next Steps.** (1) Compare root-only split vs shallow PV-split to quantify split-point effects on load imbalance. (2) Move ordering optimizations (MVV-LVA, killer moves) to reduce search tree size.

---
*Contact: sid4mndev@gmail.com | GitHub: https://github.com/Sid4mn/devi-chess-engine*