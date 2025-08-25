# devi Chess Engine

Building a chess engine from scratch to understand parallel search algorithms and push Rust's performance boundaries.

## Inspiration & Learning Resources
- **Book**: Chess Algo - Noah Caplinger - modern algorithmic approach to chess programming and search optimization
- **Book**: Computers, chess and long-range planning - M.M. Botvinnik - foundational theory on strategic planning and evaluation from a chess grandmaster's perspective
- MIT 6.5840 & Berkeley CS267 lecture sets (distributed + parallel fundamentals)
- Research papers on Lazy SMP, Jamboree search, and transposition-table design

## Project Philosophy
**Approach:** Build it, measure it, understand the bottlenecks.

## Performance Status
![Build Status](https://github.com/Sid4mn/devi-chess-engine/workflows/CI/badge.svg)

### Parallel Performance Results
![Speedup Graph](benchmarks/speedup_hires.png)

## Reproduce Results
```bash
# Run complete benchmark suite
cargo run --release -- --benchmark

# Or use the convenience script
./scripts/threads.sh
```

### Performance Results
- **Baseline**: 165.65 searches/second (single thread)
- **Peak**: 4.77× speedup on 8 threads (Apple M1 Pro, 59.6% efficiency)
- **Sweet spot**: 3.17× speedup on 4 threads (79.2% efficiency)
- **Stability**: Median 1.414ms over 100 iterations (soak test validation)
- **Methodology**: 5 warmup + 10 measurement runs, median timing with outlier detection

**Hardware**: Apple M1 Pro (6 performance + 2 efficiency cores), lock-free parallel search via Rayon

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

# Fault tolerance testing
cargo run --release -- --threads 4 --depth 4 --inject-panic 0

# Comprehensive fault analysis
cargo run --release -- --threads 4 --depth 4 --inject-panic 0 --dump-crashes
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

# Fault tolerance automation
./scripts/run_fault.sh
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
| `--inject-panic` | Inject panic at specific move index | - |
| `--dump-crashes` | Enable crash logging and analysis | false |

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
- [x] Perft validation suite (perfect through depth 7)
- [x] **Alpha-beta search implementation**
- [x] **Material evaluation function**
- [x] **CI/CD pipeline with regression tests**
- [x] **Flamegraph profiling**

## Perft Verification

| Depth | Nodes         | Status |
|-------|-------------  |------- |
| 1     | 20            |   ✅   |
| 2     | 400           |   ✅   |
| 3     | 8,902         |   ✅   |
| 4     | 197,281       |   ✅   |
| 5     | 4,865,609     |   ✅   |
| 6     | 119,060,324   |   ✅   |
| 7     | 3,195,901,860 |   ✅   |


**Week 2**: **COMPLETED** - Parallel Scalability
- [x] Root parallelization with Rayon
- [x] Multi-thread benchmarking (1/2/4/8 threads)
- [x] CLI with clap
- [x] Comprehensive benchmark suite (--benchmark flag)
- [x] Soak testing for stability validation (--soak flag)
- [x] Statistical analysis with warmup/outlier detection
- [x] Performance visualization and CSV export
- [x] **Automated reproduction scripts (threads.sh, soak.sh)**
- [x] **Speedup achievement on 8 threads**

**Week 3**: **COMPLETED** - Fault Tolerance & Distributed Systems
- [x] Fault injection mechanism via CLI flags
- [x] Panic recovery with graceful degradation
- [x] Thread-safe crash logging with JSON export
- [x] Performance overhead measurement (<12% impact)
- [x] Automated fault tolerance testing (run_fault.sh)
- [x] **Demonstrable resilience under component failure**
- [x] **Best-effort results from surviving workers**

### Fault Tolerance Results
```json
{
  "baseline": { "score": 0, "time_ms": 2.548 },
  "fault_tests": [
    { "fault_position": 0, "score": 0, "time_ms": 2.733, "overhead_percent": 7.3 },
    { "fault_position": 5, "score": 0, "time_ms": 2.840, "overhead_percent": 11.5 },
    { "fault_position": 10, "score": 0, "time_ms": 2.468, "overhead_percent": -3.1 },
    { "fault_position": 15, "score": 0, "time_ms": 2.474, "overhead_percent": -2.9 }
  ]
}
```

**Week 4**: v2 Move Ordering & Optimization
- [ ] MVV-LVA capture ordering
- [ ] Killer move heuristic
- [ ] Node reduction metrics

**Week 5**: v3 Iterative Deepening
- [ ] Time management
- [ ] Principal variation table
- [ ] Playable CLI interface

**Week 6**: Cache & Memory Studies
- [ ] Transposition table experiments
- [ ] Cache miss analysis with perf
- [ ] Memory optimization

## Current Status
**Week 3 - Complete** - Fault tolerance implementation with panic recovery and performance measurement

## Release History
- **v0.2.3-fault**: Fault tolerance implementation with panic recovery
- **v0.2.2**: Parallel search optimization and benchmarking suite
- **v0.2.1**: Core engine with perft validation

## Contributing
This is primarily a learning project, but suggestions and discussions are welcome!