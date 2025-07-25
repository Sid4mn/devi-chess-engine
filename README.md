# devi Chess Engine

A chess engine written in Rust to understand chess engine algorithms, their history, and explore intuitive Rust implementations. 
This project explores chess engine algorithms through the lens of high-performance computing, demonstrating system-level optimizations and clean Rust development.

## Project Philosophy
**Hypothesis → Experiment → Measure → Analyze → Document**

## Inspiration & Learning Resources
- **Book**: Chess Algo - Noah Caplinger - modern algorithmic approach to chess programming and search optimization
- **Book**: Computers, chess and long-range planning - M.M. Botvinnik - foundational theory on strategic planning and evaluation from a chess grandmaster's perspective
- MIT 6.5840 & Berkeley CS267 lecture sets (distributed + parallel fundamentals)
- Brendan Gregg flamegraph toolkit for CPU hotspot profiling
- Research papers on Lazy SMP, Jamboree search, and transposition-table design


### Current Implementation
- **Board Representation**: Trait-based abstraction ready for bitboard optimization
- **Move Generation**: Complete implementation for all piece types with comprehensive testing
- **Search Foundation**: Alpha-beta pruning with iterative deepening (in progress)
- **Instrumentation**: Built-in performance counters and profiling hooks (TBD)

## Features (planned)

### v1 - Baseline Alpha–Beta
- Array board + piece lists
- Material evaluation + piece-square-table evaluation
- Fixed-depth alpha–beta (depth 4)
- **Baseline metrics**: nodes/second, single-thread flamegraph

### v2 - Alpha-Beta Pruning  
- Add alpha-beta pruning to v1
- **Expected**: >8x node reduction
- **Metrics**: node count comparison, effective branching factor

### v3 - Move Ordering
- Captures-first ordering
- Killer move heuristic
- **Expected**: Additional 2-3x node reduction
- **Metrics**: beta-cutoff percentage, move ordering efficiency

### v4 - Iterative Deepening (MVP)
- Time-based search
- Principal variation reuse
- **Target**: Playable strength, responsive moves
- **Metrics**: time-to-depth, move stability

### v5 - Cache Analysis (Stretch)
- Dummy transposition table experiments
- **HPC focus**: L3 cache miss analysis
- **Metrics**: Cache misses vs table size graphs

### v6 - Parallel Search (Stretch)
- Root-level parallelization with Rayon
- **HPC focus**: Multi-core speedup
- **Metrics**: Speedup vs core count, efficiency


## Weekly Deliverables

**Week 1**: Complete move generation + legality validation
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
- [x] Perft validation suite
- [ ] Single-threaded performance baseline (needs benchmarking framework)

**Week 2**: v1 Minimax engine
- [ ] Material + PST evaluation
- [ ] Fixed-depth minimax
- [ ] First flamegraph
- [ ] Baseline nodes/second

**Week 3**: v2 Alpha-Beta
- [ ] Alpha-beta implementation
- [ ] Node reduction verification
- [ ] Tracing instrumentation

**Week 4**: v3 Move Ordering
- [ ] Capture ordering
- [ ] Killer moves
- [ ] Criterion comparison

**Week 5**: v4 Iterative Deepening
- [ ] Time management
- [ ] Playable CLI interface
- [ ] PV table

**Week 6**: Cache studies
- [ ] TT size experiments
- [ ] perf cache analysis

**Week 7**: Polish/Stretch
- [ ] Quiescence search
- [ ] Root parallelization

**Week 8**: Documentation
- [ ] Final benchmarks
- [ ] Research write-up

### Performance Metrics
- Perft accuracy
- Move generation speed (moves/second)
- Memory usage baseline

## Current Status
**Week 1 - Move Generation Phase**

## Contributing
This is primarily a learning project, but suggestions and discussions are welcome!