pub mod timer;
pub mod statistics;
pub mod harness;

pub use harness::{BenchmarkConfig, BenchmarkResult, run_benchmark};
pub use statistics::BenchmarkStats;