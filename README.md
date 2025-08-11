# devi Chess Engine

A chess engine written in Rust to understand chess engine algorithms, their history, and explore intuitive Rust implementations. 
This project explores chess engine algorithms through the lens of high-performance computing, demonstrating system-level optimizations and clean Rust development.

## Inspiration & Learning Resources
- **Book**: Chess Algo - Noah Caplinger - modern algorithmic approach to chess programming and search optimization
- **Book**: Computers, chess and long-range planning - M.M. Botvinnik - foundational theory on strategic planning and evaluation from a chess grandmaster's perspective
- MIT 6.5840 & Berkeley CS267 lecture sets (distributed + parallel fundamentals)
- Research papers on Lazy SMP, Jamboree search, and transposition-table design

## Project Philosophy
**Hypothesis → Experiment → Measure → Analyze → Document**

## Performance Status
![Build Status](https://github.com/Sid4mn/devi-chess-engine/workflows/CI/badge.svg)

### Parallel Performance Results
![Speedup Graph](benchmarks/speedup_hires.png)

## Reproduce Results
```bash
# Run complete benchmark suite
cargo run --release -- --benchmark

# Or use the convenience script
./scripts/reproduce_results.sh
```

### Performance Characteristics
- **Single-thread baseline**: 159.99 searches/second
- **Parallel scaling**: 4.69x speedup on 8 threads (Apple M1 Pro)
- **Operational range**: Depth 4-6 for optimal performance
- **Validation**: Soak testing shows consistent stability

### CLI Usage
```bash
# Run benchmark suite
cargo run --release -- --benchmark

# Single search
cargo run --release -- --threads 8 --depth 6

# Stability testing  
cargo run --release -- --soak --threads 8 --depth 6 --runs 100

# Perft testing (move generation validation)
cargo run --release -- --perft --depth 6

# Parallel perft testing
cargo run --release -- --perft --parallel-perft --threads 8 --depth 6

# Perft divide (debug individual moves)
cargo run --release -- --perft --perft-divide --depth 5
```

### Advanced Options
```bash
# Benchmark with custom parameters
cargo run --release -- --benchmark --warmup 10 --runs 20 --depth 5

# Soak test with detailed statistics
cargo run --release -- --soak --threads 4 --depth 4 --runs 50

# Serial vs Parallel perft comparison
cargo run --release -- --perft --threads 1 --depth 7 # Serial
cargo run --release -- --perft --parallel-perft --threads 8 --depth 7 # Parallel
```

### Flag Reference
| Flag | Description | Default |
|------|-------------|---------|
| `--threads` | Number of threads to use | 1 |
| `--depth` | Search depth | 4 |
| `--warmup` | Warmup iterations for benchmarks | 5 |
| `--runs` | Number of measurement runs | 10 |
| `--benchmark` | Run full benchmark suite | - |
| `--soak` | Run stability soak test | - |
| `--perft` | Run perft move generation test | - |
| `--parallel-perft` | Use parallel perft computation | false |
| `--perft-divide` | Show perft results per root move | - |

## Weekly Deliverables

**Week 1**: **COMPLETED** - Foundation & Correctness
- [x] Board representation
- [x] All piece move generation
  - [x] Pawns (forward, double, captures, en passant)
  - [x] Knights (L-shaped moves with boundary checking)
  - [x] Kings (8 adjacent squares)
  - [x] Rooks (sliding horizontal/vertical)
  - [x] Bishops (sliding diagonal)
  - [x] Queens (rook + bishop combined)
- [x] Trait-based architecture
- [x] Legal move filtering with check detection
- [x] Perft validation suite (perfect through depth 6)
- [x] **Alpha-beta search implementation**
- [x] **Material evaluation function**
- [x] **CI/CD pipeline with regression tests**
- [x] **Performance baseline: 153.48 searches/second**
- [x] **Flamegraph profiling**

## Perft Verification

| Depth | Nodes       | Status |
|-------|-------------|--------|
| 1     | 20          | ✅     |
| 2     | 400         | ✅     |
| 3     | 8,902       | ✅     |
| 4     | 197,281     | ✅     |
| 5     | 4,865,609   | ✅     |
| 6     | 119,060,324 | ✅     |

**Week 2**: **COMPLETED** - Parallel Scalability
- [x] Lazy-SMP root parallelization with Rayon
- [x] Multi-thread benchmarking (1/2/4/8 threads)
- [x] CLI with clap
- [x] Comprehensive benchmark suite (--benchmark flag)
- [x] Soak testing for stability validation (--soak flag)
- [x] Statistical analysis with warmup/outlier detection
- [x] Performance visualization and CSV export
- [x] **4.69x speedup achievement on 8 threads**

**Week 3**: v2 Move Ordering & Optimization
- [ ] MVV-LVA capture ordering
- [ ] Killer move heuristic
- [ ] Node reduction metrics

**Week 4**: v3 Iterative Deepening
- [ ] Time management
- [ ] Principal variation table
- [ ] Playable CLI interface

**Week 5**: Cache & Memory Studies
- [ ] Transposition table experiments
- [ ] Cache miss analysis with perf
- [ ] Memory optimization

### Performance Metrics

### Current Achievements
- **Search Speed**: 159.99 searches/second (single-thread baseline)
- **Parallel Scaling**: 4.69x speedup on 8 threads (Apple M1 Pro)
- **Search Depth**: 4-6 plies optimal operational range
- **Evaluation**: Material-only with clean extensibility
- **Hardware**: Apple M1 Pro (8-core, performance/efficiency hybrid)

### Statistical Validation
- **Benchmark methodology**: 5 warmup + 10 measurement runs
- **Soak testing**: 25+ iteration stability validation
- **Statistical metrics**: Min/median/p95/max timing analysis
- **Thread safety**: Zero data races in parallel execution


## Current Status
**Week 2 - Complete**

## Contributing
This is primarily a learning project, but suggestions and discussions are welcome!