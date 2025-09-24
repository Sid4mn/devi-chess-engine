use crate::benchmark::statistics::BenchmarkStats;
use crate::benchmark::timer::time_execution_millis;
use crate::board::{Board, BoardRepresentation};
use crate::scheduling::CorePolicy;
use crate::search::parallel::parallel_search_with_policy;
use crate::search::{search};

#[derive(Clone)]
pub struct BenchmarkConfig {
    pub depth: u32,
    pub warmup_runs: usize,
    pub measurement_runs: usize,
    pub thread_counts: Vec<usize>,
    pub core_policy: CorePolicy,
    pub mixed_ratio: f32, // 0.75 = 6P+2E M1 pro ratio
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        BenchmarkConfig {
            depth: 4,
            warmup_runs: 5,
            measurement_runs: 10,
            thread_counts: vec![1, 2, 4, 8],
            core_policy: CorePolicy::None,
            mixed_ratio: 0.75,
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
    pub core_policy: CorePolicy,
}

pub fn run_benchmark_with_policy(config: &BenchmarkConfig) -> Vec<BenchmarkResult> {
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

        let stats = benchmark_thread_config_with_policy(
            thread_count,
            config,
            config.core_policy,
            config.mixed_ratio,
        );
        let sps = stats.searches_per_second();

        if thread_count == 1 || (thread_count == config.thread_counts[0]) {
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
            core_policy: config.core_policy,
        });
    }

    println!("\n=== BENCHMARK COMPLETE ===");
    print_summary(&results);

    results
}

pub fn run_benchmark(config: &BenchmarkConfig) -> Vec<BenchmarkResult> {
    run_benchmark_with_policy(config)
}

fn benchmark_thread_config_with_policy(
    thread_count: usize,
    config: &BenchmarkConfig,
    policy: CorePolicy,
    mixed_ratio: f32,
) -> BenchmarkStats {
    let mut board = Board::new();

    // Warmup phase
    println!("  Warming up...");
    for _ in 0..config.warmup_runs {
        board.setup_starting_position();
        let _ =
            execute_search_with_policy(&mut board, config.depth, thread_count, policy, mixed_ratio);
    }

    // Measurement phase
    println!("  Measuring...");
    let mut samples = Vec::new();

    for run in 1..=config.measurement_runs {
        board.setup_starting_position();

        let (_, duration_ms) = time_execution_millis(|| {
            execute_search_with_policy(&mut board, config.depth, thread_count, policy, mixed_ratio)
        });

        samples.push(duration_ms);
        println!("    Run {:2}: {:.3}ms", run, duration_ms);
    }

    BenchmarkStats::from_samples(&samples)
}

fn benchmark_thread_config(thread_count: usize, config: &BenchmarkConfig) -> BenchmarkStats {
    benchmark_thread_config_with_policy(thread_count, config, CorePolicy::None, 0.0)
}

fn execute_search_with_policy(
    board: &mut Board,
    depth: u32,
    thread_count: usize,
    policy: CorePolicy,
    mixed_ratio: f32,
) -> (crate::types::Move, i32) {
    if thread_count == 1 {
        search(board, depth)
    } else {
        // QoS hints applied here via policy-specific thread pools
        parallel_search_with_policy(board, depth, policy, thread_count, mixed_ratio)
    }
}

fn execute_search(board: &mut Board, depth: u32, thread_count: usize) -> (crate::types::Move, i32) {
    execute_search_with_policy(board, depth, thread_count, CorePolicy::None, 0.0)
}

fn print_summary_with_policy(results: &[BenchmarkResult]) {
    println!("\n=== PERFORMANCE SUMMARY ===");
    println!("Threads |      Policy     | Median Time | Searches/sec | Speedup | Efficiency");
    println!("--------|-----------------|-------------|--------------|---------|-----------");

    for result in results {
        let policy_name = match result.core_policy {
            CorePolicy::None => "None",
            CorePolicy::FastBias => "FastBias", 
            CorePolicy::EfficientBias => "EfficientBias",
            CorePolicy::Mixed => "Mixed",
        };
        println!(
            "{:7} | {:11} |{:10.3}ms | {:12.2} | {:7.2}x | {:9.1}%",
            result.thread_count,
            policy_name,
            result.stats.median,
            result.searches_per_second,
            result.speedup,
            result.efficiency
        );
    }

    // TODO: CSV export for plotting
    // TODO: Statistical significance between policies
}

fn print_summary(results: &[BenchmarkResult]) {
    print_summary_with_policy(results);
}
