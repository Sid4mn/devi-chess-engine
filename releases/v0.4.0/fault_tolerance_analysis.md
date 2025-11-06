# Fault Tolerance Analysis

## Problem
Hardware faults in parallel chess search cause complete loss of in-flight work. Need to quantify recovery overhead without checkpointing.

## Method
- **Fault injection**: Panic after 2-ply evaluation at move 5 (real work wasted)
- **Recovery**: Wrapper catches panic, retries entire search from root
- **Hardware**: M1 Pro (8P+2E), 8 threads, depth 7

## Results

| Scenario | Time (ms) | Overhead |
|----------|-----------|----------|
| Baseline | 417 | 0% |
| Wrapper only | 437 | +5% |
| **With panic** | **834** | **+100%** |
| Double work | 885 | +112% |

## Why 100%?
Rayon stops ALL workers on ANY panic:
1. 8 threads evaluate moves in parallel (~450ms)
2. Thread panics at move 5 → Rayon kills all workers
3. ~450ms parallel work discarded
4. Full retry: ~450ms
5. Total: 2× baseline

Double-work validation (112%) confirms measurement accuracy.

## Key Insight
`par_iter().collect()` requires all results or none - no partial recovery possible without custom thread pool.

## Future Work
**Checkpointing** (target: 15-30% overhead):
```rust
checkpoint.save(completed_moves[0..4]);
retry(remaining_moves[5..]);  // Skip checkpointed
```

## Reproduce
```bash
cargo build --release
./target/release/devi --fault-analysis --depth 7 --threads 8
# Output: benchmarks/fault_overhead.csv
```

## Significance
First baseline measurement of fault tolerance overhead in parallel alpha-beta search. Establishes upper bound for checkpoint-based recovery research.
