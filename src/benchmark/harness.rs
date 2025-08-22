use crate::benchmark::statistics::BenchmarkStats;
use crate::benchmark::timer::time_execution_millis;
use crate::board::{Board, BoardRepresentation};
use crate::search::{parallel_search, search};

#[derive(Clone)]
pub struct BenchmarkConfig {
    pub depth: u32,
    pub warmup_runs: usize,
    pub measurement_runs: usize,
    pub thread_counts: Vec<usize>,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        BenchmarkConfig {
            depth: 4,
            warmup_runs: 5,
            measurement_runs: 10,
            thread_counts: vec![1, 2, 4, 8],
        }
    }
}

#[derive(Debug)]
pub struct BenchmarkResult {
    pub thread_count: usize,
    pub stats: BenchmarkStats,
    pub searches_per_second: f64,
    pub speedup: f64,
    pub efficiency: f64,
}

pub fn run_benchmark(config: &BenchmarkConfig) -> Vec<BenchmarkResult> {
    println!("\n=== UNIFIED CHESS ENGINE BENCHMARK ===");
    println!("Configuration:");
    println!("  Depth: {}", config.depth);
    println!("  Warmup runs: {}", config.warmup_runs);
    println!("  Measurement runs: {}", config.measurement_runs);
    println!("  Thread configurations: {:?}", config.thread_counts);

    let mut results = Vec::new();
    let mut baseline_sps = 0.0;

    for &thread_count in &config.thread_counts {
        println!("\n--- Testing {} thread(s) ---", thread_count);

        let stats = benchmark_thread_config(thread_count, config);
        let sps = stats.searches_per_second();

        if thread_count == 1 {
            baseline_sps = sps;
        }

        let speedup = if baseline_sps > 0.0 {
            sps / baseline_sps
        } else {
            1.0
        };
        let efficiency = speedup / thread_count as f64 * 100.0;

        println!(
            "  Median: {:.3}ms (std dev: {:.3}ms)",
            stats.median, stats.std_dev
        );
        println!("  Range: {:.3}ms - {:.3}ms", stats.min, stats.max);
        println!("  Searches/second: {:.2}", sps);
        println!("  Speedup: {:.2}x", speedup);
        println!("  Efficiency: {:.1}%", efficiency);

        results.push(BenchmarkResult {
            thread_count,
            stats,
            searches_per_second: sps,
            speedup,
            efficiency,
        });
    }

    println!("\n=== BENCHMARK COMPLETE ===");
    print_summary(&results);

    results
}

fn benchmark_thread_config(thread_count: usize, config: &BenchmarkConfig) -> BenchmarkStats {
    let mut board = Board::new();

    // Warmup phase
    println!("  Warming up...");
    for _ in 0..config.warmup_runs {
        board.setup_starting_position();
        let _ = execute_search(&mut board, config.depth, thread_count);
    }

    // Measurement phase
    println!("  Measuring...");
    let mut samples = Vec::new();

    for run in 1..=config.measurement_runs {
        board.setup_starting_position();

        let (_, duration_ms) =
            time_execution_millis(|| execute_search(&mut board, config.depth, thread_count));

        samples.push(duration_ms);
        println!("    Run {:2}: {:.3}ms", run, duration_ms);
    }

    BenchmarkStats::from_samples(&samples)
}

fn execute_search(board: &mut Board, depth: u32, thread_count: usize) -> (crate::types::Move, i32) {
    if thread_count == 1 {
        search(board, depth)
    } else {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(thread_count)
            .build()
            .expect("Failed to create thread pool");

        pool.install(|| parallel_search(board, depth))
    }
}

fn print_summary(results: &[BenchmarkResult]) {
    println!("\n=== PERFORMANCE SUMMARY ===");
    println!("Threads | Median Time | Searches/sec | Speedup | Efficiency");
    println!("--------|-------------|--------------|---------|------------");

    for result in results {
        println!(
            "{:7} | {:10.3}ms | {:12.2} | {:7.2}x | {:9.1}%",
            result.thread_count,
            result.stats.median,
            result.searches_per_second,
            result.speedup,
            result.efficiency
        );
    }
}
