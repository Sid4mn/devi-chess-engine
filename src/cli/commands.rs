use crate::benchmark::{run_benchmark, BenchmarkConfig, BenchmarkResult};
use crate::board::{Board, BoardRepresentation};
use crate::cli::Cli;
use crate::evaluation::evaluate;
use crate::moves::{perft, perft_divide, perft_parallel};
use crate::scheduling::CorePolicy;
use crate::search::fault_tolerant::with_recovery;
use crate::search::parallel::parallel_search_with_fault;
use crate::search::parallel::parallel_search_with_policy;
use crate::search::probe::ClassificationConfig;
use crate::search::{parallel_search, search, two_phase_search, two_phase_search_with_metrics, TwoPhaseConfig, TwoPhaseMetrics}; 
use rayon;
use std::fs::{create_dir_all, File};
use std::io::{BufWriter, Write};
use std::time::Instant;

struct FaultMeasurement {
    scenario: &'static str,
    times_ms: Vec<f64>,
    median_ms: f64,
    best_move: String,
    score: i32,
}

pub fn run_full_benchmark(args: &Cli) {
    let mut board = Board::new();
    board.setup_starting_position();
    println!("Starting position evaluation: {}", evaluate(&board));

    let policy = args.core_policy.unwrap_or(CorePolicy::None);
    let mixed_ratio = args.mixed_ratio;

    let threads = {
        if args.benchmark_sweep {
            vec![1, 2, 4, 6, 8, 10]
        } else {
            vec![args.threads]
        }
    };

    let config = BenchmarkConfig {
        depth: args.depth,
        warmup_runs: args.warmup,
        measurement_runs: args.runs,
        thread_counts: threads,
        core_policy: policy,
        mixed_ratio: mixed_ratio,
        inject_panic: args.inject_panic,
    };

    println!("Core scheduling policy: {:?}", policy);
    if matches!(policy, CorePolicy::Mixed) {
        println!(
            "Mixed ratio: {:.2} ({}% fast cores)",
            mixed_ratio,
            (mixed_ratio * 100.0) as u32
        );
    }

    let results = run_benchmark(&config);
    export_benchmark_csv(&results, args.csv_output.as_deref());
}

pub fn run_single_search(args: &Cli) {
    let mut board = Board::new();
    board.setup_starting_position();

    println!("Starting position evaluation: {}", evaluate(&mut board));
    println!("Searching to depth {}...", args.depth);

    let policy = args.core_policy.unwrap_or(CorePolicy::None);
    let mixed_ratio = args.mixed_ratio;

    if let Some(ref p) = args.core_policy {
        println!("Using core policy: {:?}", p);
    }

    if args.two_phase {
        let config = TwoPhaseConfig {
            probe_depth: args.probe_depth,
            p_core_threads: args.p_cores,
            e_core_threads: args.e_cores,
            classification: ClassificationConfig {
                heavy_ratio: args.heavy_ratio,
                light_threshold: args.light_threshold,
            },
        };
        
        println!("Using two-phase scheduler:");
        println!("  Probe depth: {}", config.probe_depth);
        println!("  P-cores: {}", config.p_core_threads);
        println!("  E-cores: {}", config.e_core_threads);
        println!("  Heavy ratio: {:.1}", config.classification.heavy_ratio);
        
        let start = Instant::now();
        let (best_move, score) = two_phase_search(&mut board, args.depth, &config);
        let elapsed = start.elapsed();
        
        println!("\nResult:");
        println!("  Best move: {}", best_move.to_algebraic());
        println!("  Score: {}", score);
        println!("  Time: {:.3}ms", elapsed.as_secs_f64() * 1000.0);
        return;
    }

    // Wrap with recovery if panic injection requested
    let start = Instant::now();
    let (best_move, score) = if args.inject_panic.is_some() {
        println!("Fault injection enabled at move {:?}", args.inject_panic);

        let search_fn = || {
            let mut b = board.clone();
            if args.threads == 1 {
                search(&mut b, args.depth)
            } else {
                parallel_search_with_policy(&mut b, args.depth, policy, args.threads, mixed_ratio)
            }
        };
        with_recovery(search_fn, args.inject_panic)
    } else {
        if args.threads == 1 {
            search(&mut board, args.depth)
        } else {
            parallel_search_with_policy(&mut board, args.depth, policy, args.threads, mixed_ratio)
        }
    };
    let elapsed = start.elapsed();

    println!("\nResult:");
    println!("  Best move: {}", best_move.to_algebraic());
    println!("  Score: {}", score);
    println!("  Time: {:.3}ms", elapsed.as_secs_f64() * 1000.0);
}

pub fn run_recovery_analysis(args: &Cli) {
    println!("=== THREAD RECOVERY ANALYSIS ===");
    println!("Testing retry-based recovery with panic injection\n");

    let mut board = Board::new();
    board.setup_starting_position();

    // Test 1: Baseline (no panic)
    println!("Test 1: Baseline (no panic injection)");
    let start = Instant::now();
    let (mv1, score1) = parallel_search(&mut board, args.depth);
    let time1 = start.elapsed();
    println!(
        "Move: {}, Score: {}, Time: {:.3}ms",
        mv1,
        score1,
        time1.as_secs_f64() * 1000.0
    );

    // Test 2: With panic + recovery
    println!("\nTest 2: With panic injection + recovery");
    let start = Instant::now();

    // Clone board for recovery closure
    let board_clone = board.clone();
    let search_fn = || {
        let mut b = board_clone.clone();
        parallel_search(&mut b, args.depth)
    };

    let (mv2, score2) = with_recovery(search_fn, Some(5));
    let time2 = start.elapsed();
    println!(
        "Move: {}, Score: {}, Time: {:.3}ms",
        mv2,
        score2,
        time2.as_secs_f64() * 1000.0
    );

    // Verify correctness
    println!("\n=== CORRECTNESS CHECK ===");
    if mv1.to_algebraic() != mv2.to_algebraic() {
        println!("WARNING: Move changed! {} -> {}", mv1, mv2);
    } else {
        println!("Move preserved: {}", mv1);
    }

    if score1 != score2 {
        println!("WARNING: Score changed! {} -> {}", score1, score2);
    } else {
        println!("Score preserved: {}", score1);
    }

    let overhead = ((time2.as_millis() as f64 / time1.as_millis() as f64) - 1.0) * 100.0;
    println!("\nRecovery overhead: {:.1}%", overhead);
}

pub fn run_soak_test(args: &Cli) {
    use std::time::Instant;

    println!("--- SOAK TEST ---");
    println!(
        "Threads: {}, Depth: {}, Iterations: {}",
        args.threads, args.depth, args.runs
    );

    let mut samples_ms: Vec<f64> = Vec::new();

    for i in 1..=args.runs {
        let mut board = Board::new();
        board.setup_starting_position();

        let start = Instant::now();
        let _ = if args.threads == 1 {
            search(&mut board, args.depth)
        } else {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(args.threads)
                .build()
                .expect("Failed to create thread pool");
            pool.install(|| parallel_search(&mut board, args.depth))
        };

        let duration_ms = start.elapsed().as_micros() as f64 / 1000.0;
        samples_ms.push(duration_ms);
        println!("Run {:3}: {:.3}ms", i, duration_ms);
    }
    //calculating stats.
    samples_ms.sort_by(|a, b| a.partial_cmp(b).unwrap());

    if samples_ms.len() > 0 {
        let len = samples_ms.len();
        let min = samples_ms[0];
        let max = samples_ms[len - 1];
        let median = if len % 2 == 0 {
            (samples_ms[len / 2 - 1] + samples_ms[len / 2]) / 2.0
        } else {
            samples_ms[len / 2]
        };

        let p95_idx = ((len as f64 * 0.95) as usize).min(len - 1);
        let p95 = samples_ms[p95_idx];
        println!(
            "Summary: min {:.3}ms, median {:.3}ms, p95 {:.3}ms, max {:.3}ms",
            min, median, p95, max
        );

        write_soak_files(
            &samples_ms,
            args.threads,
            args.depth,
            args.runs,
            min,
            median,
            p95,
            max,
        );
    } else {
        println!("No samples collected!");
    }
}

fn write_soak_files(
    samples: &[f64],
    threads: usize,
    depth: u32,
    runs: usize,
    min: f64,
    median: f64,
    p95: f64,
    max: f64,
) {
    // Ensure docs directory exists
    if let Err(e) = create_dir_all("docs") {
        eprintln!("Warning: Failed to create docs directory: {}", e);
        return;
    }

    // Write raw samples
    if let Err(e) = write_raw_samples(samples) {
        eprintln!("Warning: Failed to write raw samples: {}", e);
    }

    // Write summary
    if let Err(e) = write_soak_summary(threads, depth, runs, min, median, p95, max) {
        eprintln!("Warning: Failed to write summary: {}", e);
    }

    println!("\nSoak test results written to docs/soak_raw.txt and docs/soak_summary.txt");
}

fn write_raw_samples(samples: &[f64]) -> std::io::Result<()> {
    let file = File::create("docs/soak_raw.txt")?;
    let mut writer = BufWriter::new(file);

    for sample in samples {
        writeln!(writer, "{:.3}", sample)?;
    }

    writer.flush()?;
    Ok(())
}

fn write_soak_summary(
    threads: usize,
    depth: u32,
    runs: usize,
    min: f64,
    median: f64,
    p95: f64,
    max: f64,
) -> std::io::Result<()> {
    let file = File::create("docs/soak_summary.txt")?;
    let mut writer = BufWriter::new(file);

    writeln!(writer, "Devi Chess Engine - Soak Test Results")?;
    writeln!(writer, "=====================================")?;
    writeln!(writer)?;
    writeln!(writer, "Configuration:")?;
    writeln!(writer, "  Threads: {}", threads)?;
    writeln!(writer, "  Depth: {}", depth)?;
    writeln!(writer, "  Iterations: {}", runs)?;
    writeln!(writer)?;
    writeln!(writer, "Performance Statistics (milliseconds):")?;
    writeln!(writer, "  Minimum:  {:.3}ms", min)?;
    writeln!(writer, "  Median:   {:.3}ms", median)?;
    writeln!(writer, "  95th %:   {:.3}ms", p95)?;
    writeln!(writer, "  Maximum:  {:.3}ms", max)?;

    writer.flush()?;
    Ok(())
}

pub fn run_perft_test(args: &Cli) {
    println!("--- PERFT TEST ---");
    let parallel = args.parallel_perft;
    println!(
        "Mode: {} (threads: {})",
        if parallel { "Parallel" } else { "Serial" },
        args.threads
    );

    let mut board = Board::new();
    board.setup_starting_position();

    if args.perft_divide && args.depth > 0 {
        println!("\n--- PERFT DIVIDE at depth {} ---", args.depth);
        let (results, total) = perft_divide(&mut board, args.depth);
        for (move_str, count) in &results {
            println!("{}: {}", move_str, count);
        }
        println!("Total: {}", total);
        return;
    }

    //Set up thread pool if parallel
    if parallel {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(args.threads)
            .build()
            .expect("Failed to create thread pool");

        pool.install(|| run_perft_depths(args, &mut board, parallel));
    } else {
        run_perft_depths(args, &mut board, parallel);
    }
}

pub fn run_perft_depths(args: &Cli, board: &mut Board, parallel: bool) {
    println!("\nDepth | Nodes        | Time     | Nodes/sec");
    println!("------|--------------|----------|----------");

    for depth in 1..=args.depth {
        let start = Instant::now();

        let nodes = if parallel {
            perft_parallel(board, depth)
        } else {
            perft(board, depth)
        };

        let elapsed = start.elapsed();
        let nps = nodes as f64 / elapsed.as_secs_f64();

        println!(
            "{:>4} | {:>12} | {:>8.2}s | {:>10.0}",
            depth,
            format_with_commas(nodes),
            elapsed.as_secs_f64(),
            nps
        );
    }
}

fn format_with_commas(n: u64) -> String {
    let s: String = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

pub fn export_benchmark_csv_with_policy(results: &[BenchmarkResult], custom_path: Option<&str>) {
    let csv_path = custom_path.unwrap_or("benchmarks/speedup.csv");

    // Create directory if needed
    if let Some(parent) = std::path::Path::new(csv_path).parent() {
        if !parent.exists() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                eprintln!(
                    "Warning: Failed to create directory {}: {}",
                    parent.display(),
                    e
                );
                return export_benchmark_csv_with_policy(results, None);
            }
        }
    }

    let mut file = match std::fs::File::create(csv_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error: Failed to create {}: {}", csv_path, e);
            panic!("Cannot write benchmark results");
        }
    };

    let _timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Or use a more readable format:
    let dt = chrono::Local::now();
    let timestamp_str = dt.format("%Y-%m-%d_%H:%M:%S").to_string();

    // Write header with timestamp as first column
    writeln!(
        file,
        "timestamp,threads,policy,median_ms,searches_per_sec,speedup,efficiency"
    )
    .unwrap();

    // Write data rows
    for result in results {
        writeln!(
            file,
            "{},{},{:?},{:.3},{:.2},{:.2},{:.1}",
            timestamp_str, // Add timestamp to each row
            result.thread_count,
            result.core_policy,
            result.stats.median,
            result.searches_per_second,
            result.speedup,
            result.efficiency
        )
        .unwrap();
    }

    println!(
        "\nBenchmark results written to {} (timestamp: {})",
        csv_path, timestamp_str
    );
}

pub fn export_benchmark_csv(results: &[BenchmarkResult], custom_path: Option<&str>) {
    export_benchmark_csv_with_policy(results, custom_path);
}

pub fn run_fault_overhead_analysis(args: &Cli) {
    let depth = if args.depth < 7 { 7 } else { args.depth };
    let threads = args.threads;
    let iterations = 5;
    let warmup_per_scenario = 3;

    println!("Fault Tolerance Overhead Analysis");
    println!(
        "Depth: {}, Threads: {}, Iterations: {}\n",
        depth, threads, iterations
    );

    let mut board = Board::new();
    board.setup_starting_position();

    let mut results = Vec::new();

    // === Baseline ===
    println!("Scenario 1: Baseline (no recovery wrapper)");
    println!("  Warming up ({} runs)...", warmup_per_scenario);
    for _ in 0..warmup_per_scenario {
        let mut b = board.clone();
        let _ = if threads == 1 {
            search(&mut b, depth)
        } else {
            parallel_search(&mut b, depth)
        };
    }

    println!("  Measuring...");
    let mut baseline_times = Vec::new();
    let mut baseline_move = String::new();
    let mut baseline_score = 0;

    for i in 1..=iterations {
        let mut b = board.clone();
        let start = Instant::now();
        let (mv, score) = if threads == 1 {
            search(&mut b, depth)
        } else {
            parallel_search(&mut b, depth)
        };
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        baseline_times.push(elapsed);
        baseline_move = mv.to_algebraic();
        baseline_score = score;
        println!("    Run {}: {:.3}ms", i, elapsed);
    }
    baseline_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let baseline_median = baseline_times[baseline_times.len() / 2];
    println!("  Median: {:.3}ms\n", baseline_median);

    results.push(FaultMeasurement {
        scenario: "baseline",
        times_ms: baseline_times,
        median_ms: baseline_median,
        best_move: baseline_move.clone(),
        score: baseline_score,
    });

    // === Zero-overhead check ===
    println!("Scenario 2: Zero-overhead (wrapper, no panic)");
    println!("  Warming up ({} runs)...", warmup_per_scenario);
    for _ in 0..warmup_per_scenario {
        let board_clone = board.clone();
        let search_fn = || {
            let mut b = board_clone.clone();
            if threads == 1 {
                search(&mut b, depth)
            } else {
                parallel_search(&mut b, depth)
            }
        };
        let _ = with_recovery(search_fn, None);
    }

    println!("  Measuring...");
    let mut zero_times = Vec::new();
    let mut zero_move = String::new();
    let mut zero_score = 0;

    for i in 1..=iterations {
        let board_clone = board.clone();
        let search_fn = || {
            let mut b = board_clone.clone();
            if threads == 1 {
                search(&mut b, depth)
            } else {
                parallel_search(&mut b, depth)
            }
        };
        let start = Instant::now();
        let (mv, score) = with_recovery(search_fn, None);
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        zero_times.push(elapsed);
        zero_move = mv.to_algebraic();
        zero_score = score;
        println!("    Run {}: {:.3}ms", i, elapsed);
    }
    zero_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let zero_median = zero_times[zero_times.len() / 2];
    let zero_overhead = ((zero_median - baseline_median) / baseline_median) * 100.0;
    println!("  Median: {:.3}ms", zero_median);
    println!("  Overhead: {:.2}%\n", zero_overhead);

    results.push(FaultMeasurement {
        scenario: "zero_overhead",
        times_ms: zero_times,
        median_ms: zero_median,
        best_move: zero_move,
        score: zero_score,
    });

    // === Recovery with actual fault ===
    println!("Scenario 3: With panic (recovery triggered)");
    println!("  Warming up ({} runs)...", warmup_per_scenario);
    for _ in 0..warmup_per_scenario {
        let board_clone = board.clone();
        let search_fn = || {
            let mut b = board_clone.clone();
            if threads == 1 {
                search(&mut b, depth)
            } else {
                parallel_search_with_fault(&mut b, depth, CorePolicy::None, threads, 0.0, Some(5))
            }
        };
        let _ = with_recovery(search_fn, Some(5));
    }

    println!("  Measuring...");
    let mut panic_times = Vec::new();
    let mut panic_move = String::new();
    let mut panic_score = 0;

    for i in 1..=iterations {
        let board_clone = board.clone();
        let search_fn = || {
            let mut b = board_clone.clone();
            if threads == 1 {
                search(&mut b, depth)
            } else {
                parallel_search_with_fault(&mut b, depth, CorePolicy::None, threads, 0.0, Some(5))
            }
        };

        let start = Instant::now();
        let (mv, score) = with_recovery(search_fn, Some(5));
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;

        panic_times.push(elapsed);
        panic_move = mv.to_algebraic();
        panic_score = score;
        println!("    Run {}: {:.3}ms (includes retry)", i, elapsed);
    }
    panic_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let panic_median = panic_times[panic_times.len() / 2];
    let panic_overhead = ((panic_median - baseline_median) / baseline_median) * 100.0;
    println!("  Median: {:.3}ms", panic_median);
    println!("  Overhead: {:.2}%\n", panic_overhead);

    results.push(FaultMeasurement {
        scenario: "with_panic",
        times_ms: panic_times,
        median_ms: panic_median,
        best_move: panic_move,
        score: panic_score,
    });

    // === Sanity: double work ===
    println!("Scenario 4: Double work (sanity check)");
    println!("  Measuring...");
    let mut double_times = Vec::new();

    for i in 1..=iterations {
        let mut b = board.clone();
        let start = Instant::now();

        let (mv1, score1) = if threads == 1 {
            search(&mut b, depth)
        } else {
            parallel_search(&mut b, depth)
        };

        let mut b2 = board.clone();
        let (_mv2, _score2) = if threads == 1 {
            search(&mut b2, depth)
        } else {
            parallel_search(&mut b2, depth)
        };

        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        double_times.push(elapsed);
        println!("    Run {}: {:.3}ms (2x work)", i, elapsed);

        assert_eq!(
            mv1.to_algebraic(),
            _mv2.to_algebraic(),
            "Non-deterministic!"
        );
        assert_eq!(score1, _score2, "Score mismatch!");
    }

    double_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let double_median = double_times[double_times.len() / 2];
    let double_overhead = ((double_median - baseline_median) / baseline_median) * 100.0;
    println!("  Median: {:.3}ms", double_median);
    println!("  Overhead: {:.2}% (expected ~100%)\n", double_overhead);

    results.push(FaultMeasurement {
        scenario: "double_work",
        times_ms: double_times,
        median_ms: double_median,
        best_move: baseline_move.clone(),
        score: baseline_score,
    });

    let csv_path = "benchmarks/fault_overhead.csv";
    export_fault_csv(&results, csv_path, depth, threads);

    println!("SUMMARY:");
    println!("  Baseline:      {:.3}ms", baseline_median);
    println!(
        "  Zero-overhead: {:.3}ms ({:+.2}%)",
        zero_median, zero_overhead
    );
    println!(
        "  With panic:    {:.3}ms ({:+.2}%)",
        panic_median, panic_overhead
    );
    println!(
        "  Double work:   {:.3}ms ({:+.2}%)",
        double_median, double_overhead
    );

    let correctness_passed = results
        .iter()
        .all(|r| r.best_move == baseline_move && r.score == baseline_score);
    println!(
        "  Correctness:   {}",
        if correctness_passed { "PASS" } else { "FAIL" }
    );
    println!("\nResults exported to: {}", csv_path);
}

fn export_fault_csv(results: &[FaultMeasurement], path: &str, depth: u32, threads: usize) {
    if let Some(parent) = std::path::Path::new(path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let mut file = std::fs::File::create(path).unwrap();
    let ts = chrono::Local::now().format("%Y-%m-%d_%H:%M:%S").to_string();

    writeln!(
        file,
        "timestamp,depth,threads,scenario,median_ms,overhead_pct,move,score,min_ms,max_ms"
    )
    .unwrap();

    let baseline_ms = results[0].median_ms;
    for r in results {
        let overhead = if r.scenario == "baseline" {
            0.0
        } else {
            ((r.median_ms - baseline_ms) / baseline_ms) * 100.0
        };

        let min_ms = r.times_ms.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_ms = r.times_ms.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        writeln!(
            file,
            "{},{},{},{},{:.3},{:.2},{},{},{:.3},{:.3}",
            ts,
            depth,
            threads,
            r.scenario,
            r.median_ms,
            overhead,
            r.best_move,
            r.score,
            min_ms,
            max_ms
        )
        .unwrap();
    }
}

/// Two-phase benchmark result for a single configuration
#[derive(Debug, Clone)]
pub struct TwoPhaseBenchmarkResult {
    pub config_name: String,
    pub position_name: String,
    pub probe_depth: u8,
    pub heavy_ratio: f32,
    pub samples: Vec<TwoPhaseMetrics>,
    pub median_total_ms: f64,
    pub median_probe_ms: f64,
    pub median_phase1_ms: f64,
    pub median_phase2_ms: f64,
    pub searches_per_second: f64,
    pub speedup_vs_baseline: f64,
}

/// Standard test positions for benchmarking
pub const BENCHMARK_POSITIONS: &[(&str, &str)] = &[
    ("starting", "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"),
    ("kiwipete", "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1"),
    ("position4", "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1"),
];

pub fn run_two_phase_benchmark(args: &Cli) {
    println!("=== TWO-PHASE SCHEDULER BENCHMARK ===\n");
    
    let depth = args.depth;
    let warmup = args.warmup;
    let runs = args.runs;
    
    // Determine positions to test
    let positions: Vec<(String, String)> = if let Some(ref fen) = args.fen {
        vec![("custom".to_string(), fen.clone())]
    } else {
        BENCHMARK_POSITIONS.iter()
            .map(|(n, f)| (n.to_string(), f.to_string()))
            .collect()
    };
    
    // Probe depths to test
    let probe_depths = vec![1, 2, 3];
    
    // Heavy ratios to test (for classification tuning)
    let heavy_ratios = vec![0.5, 0.6, 0.7, 0.8];
    
    let mut all_results: Vec<TwoPhaseBenchmarkResult> = Vec::new();
    
    for (pos_name, fen) in &positions {
        println!("Position: {} ({})", pos_name, fen);
        println!("{}", "-".repeat(70));
        
        let mut board = match Board::from_fen(fen) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("Failed to parse FEN: {}", e);
                continue;
            }
        };
        
        // Baseline: 10 threads, no scheduling
        println!("\n[Baseline] 10 threads, CorePolicy::None");
        let baseline_result = benchmark_baseline(&mut board, depth, warmup, runs, pos_name);
        let baseline_sps = baseline_result.searches_per_second;
        all_results.push(baseline_result);
        
        // Small delay to let thread pools fully deallocate
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        // FastBias: P-cores only
        println!("\n[FastBias] 8 threads, CorePolicy::FastBias");
        let fast_result = benchmark_fast_bias(&mut board, depth, warmup, runs, pos_name, baseline_sps);
        all_results.push(fast_result);
        
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        // Two-phase with probe depth sweep
        for &probe_depth in &probe_depths {
            println!("\n[TwoPhase] probe_depth={}, ratio=0.6", probe_depth);
            let tp_result = benchmark_two_phase(
                &mut board, depth, warmup, runs, pos_name,
                probe_depth, 0.6, 0.3, 
                args.p_cores, args.e_cores,
                baseline_sps
            );
            all_results.push(tp_result);
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        
        // Classification ratio sweep (with best probe depth)
        for &ratio in &heavy_ratios {
            if ratio == 0.6 { continue; } // Already tested
            println!("\n[TwoPhase] probe_depth=1, ratio={:.1}", ratio);
            let tp_result = benchmark_two_phase(
                &mut board, depth, warmup, runs, pos_name,
                1, ratio, 0.3,
                args.p_cores, args.e_cores,
                baseline_sps
            );
            all_results.push(tp_result);
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        
        println!("\n");
    }
    
    // Export CSV
    let csv_path = args.csv_output.as_deref().unwrap_or("benchmarks/v0.5.0/two_phase_benchmark.csv");
    export_two_phase_csv(&all_results, csv_path, depth);
    
    // Print summary
    print_two_phase_summary(&all_results);
}

fn benchmark_baseline(board: &mut Board, depth: u32, warmup: usize, runs: usize, pos_name: &str) -> TwoPhaseBenchmarkResult {
    // Warmup
    for _ in 0..warmup {
        let mut b = board.clone();
        let _ = parallel_search_with_policy(&mut b, depth, CorePolicy::None, 10, 0.8);
    }
    
    // Measure
    let mut samples: Vec<TwoPhaseMetrics> = Vec::new();
    for i in 1..=runs {
        let mut b = board.clone();
        let start = Instant::now();
        let (mv, score) = parallel_search_with_policy(&mut b, depth, CorePolicy::None, 10, 0.8);
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        
        let metrics = TwoPhaseMetrics {
            probe_time_ms: 0.0,
            phase1_time_ms: elapsed,
            phase2_time_ms: 0.0,
            total_time_ms: elapsed,
            heavy_move_count: 0,
            light_move_count: 0,
            best_move: mv.to_algebraic(),
            score,
        };
        samples.push(metrics);
        print!("  Run {}: {:.1}ms  ", i, elapsed);
        if i % 5 == 0 { println!(); }
    }
    if runs % 5 != 0 { println!(); }
    
    let median = median_time(&samples);
    let sps = 1000.0 / median;
    
    TwoPhaseBenchmarkResult {
        config_name: "baseline".to_string(),
        position_name: pos_name.to_string(),
        probe_depth: 0,
        heavy_ratio: 0.0,
        samples,
        median_total_ms: median,
        median_probe_ms: 0.0,
        median_phase1_ms: median,
        median_phase2_ms: 0.0,
        searches_per_second: sps,
        speedup_vs_baseline: 1.0,
    }
}

fn benchmark_fast_bias(board: &mut Board, depth: u32, warmup: usize, runs: usize, pos_name: &str, baseline_sps: f64) -> TwoPhaseBenchmarkResult {
    for _ in 0..warmup {
        let mut b = board.clone();
        let _ = parallel_search_with_policy(&mut b, depth, CorePolicy::FastBias, 8, 0.8);
    }
    
    let mut samples: Vec<TwoPhaseMetrics> = Vec::new();
    for i in 1..=runs {
        let mut b = board.clone();
        let start = Instant::now();
        let (mv, score) = parallel_search_with_policy(&mut b, depth, CorePolicy::FastBias, 8, 0.8);
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        
        let metrics = TwoPhaseMetrics {
            probe_time_ms: 0.0,
            phase1_time_ms: elapsed,
            phase2_time_ms: 0.0,
            total_time_ms: elapsed,
            heavy_move_count: 0,
            light_move_count: 0,
            best_move: mv.to_algebraic(),
            score,
        };
        samples.push(metrics);
        print!("  Run {}: {:.1}ms  ", i, elapsed);
        if i % 5 == 0 { println!(); }
    }
    if runs % 5 != 0 { println!(); }
    
    let median = median_time(&samples);
    let sps = 1000.0 / median;
    
    TwoPhaseBenchmarkResult {
        config_name: "fast_bias".to_string(),
        position_name: pos_name.to_string(),
        probe_depth: 0,
        heavy_ratio: 0.0,
        samples,
        median_total_ms: median,
        median_probe_ms: 0.0,
        median_phase1_ms: median,
        median_phase2_ms: 0.0,
        searches_per_second: sps,
        speedup_vs_baseline: sps / baseline_sps,
    }
}

fn benchmark_two_phase(board: &mut Board, depth: u32, warmup: usize, runs: usize, pos_name: &str,probe_depth: u8, heavy_ratio: f32, light_threshold: f32,p_cores: usize, e_cores: usize, baseline_sps: f64) -> TwoPhaseBenchmarkResult {
    let config = TwoPhaseConfig {
        probe_depth,
        p_core_threads: p_cores,
        e_core_threads: e_cores,
        classification: ClassificationConfig {
            heavy_ratio,
            light_threshold,
        },
    };
    
    for _ in 0..warmup {
        let mut b = board.clone();
        let _ = two_phase_search(&mut b, depth, &config);
    }
    
    let mut samples: Vec<TwoPhaseMetrics> = Vec::new();
    for i in 1..=runs {
        let mut b = board.clone();
        let (_, _, metrics) = two_phase_search_with_metrics(&mut b, depth, &config);
        samples.push(metrics.clone());
        print!("  Run {}: {:.1}ms (probe: {:.1}ms, P1: {:.1}ms, P2: {:.1}ms)  ", 
            i, metrics.total_time_ms, metrics.probe_time_ms, 
            metrics.phase1_time_ms, metrics.phase2_time_ms);
        if i % 2 == 0 { println!(); }
    }
    if runs % 2 != 0 { println!(); }
    
    let median_total = median_time(&samples);
    let median_probe = median_of(&samples.iter().map(|s| s.probe_time_ms).collect::<Vec<_>>());
    let median_p1 = median_of(&samples.iter().map(|s| s.phase1_time_ms).collect::<Vec<_>>());
    let median_p2 = median_of(&samples.iter().map(|s| s.phase2_time_ms).collect::<Vec<_>>());
    let sps = 1000.0 / median_total;
    
    TwoPhaseBenchmarkResult {
        config_name: format!("two_phase_p{}_r{}", probe_depth, (heavy_ratio * 10.0) as u8),
        position_name: pos_name.to_string(),
        probe_depth,
        heavy_ratio,
        samples,
        median_total_ms: median_total,
        median_probe_ms: median_probe,
        median_phase1_ms: median_p1,
        median_phase2_ms: median_p2,
        searches_per_second: sps,
        speedup_vs_baseline: sps / baseline_sps,
    }
}

fn median_time(samples: &[TwoPhaseMetrics]) -> f64 {
    let mut times: Vec<f64> = samples.iter().map(|s| s.total_time_ms).collect();
    times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mid = times.len() / 2;
    if times.len() % 2 == 0 {
        (times[mid - 1] + times[mid]) / 2.0
    } else {
        times[mid]
    }
}

fn median_of(values: &[f64]) -> f64 {
    if values.is_empty() { return 0.0; }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mid = sorted.len() / 2;
    if sorted.len() % 2 == 0 {
        (sorted[mid - 1] + sorted[mid]) / 2.0
    } else {
        sorted[mid]
    }
}

fn stddev_of(values: &[f64]) -> f64 {
    if values.len() < 2 { return 0.0; }
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let variance = values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (values.len() - 1) as f64;
    variance.sqrt()
}

fn export_two_phase_csv(results: &[TwoPhaseBenchmarkResult], path: &str, depth: u32) {
    if let Some(parent) = std::path::Path::new(path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    
    let mut file = std::fs::File::create(path).expect("Failed to create CSV file");
    let ts = chrono::Local::now().format("%Y-%m-%d_%H:%M:%S").to_string();
    
    writeln!(file, "timestamp,depth,position,config,probe_depth,heavy_ratio,median_total_ms,median_probe_ms,median_phase1_ms,median_phase2_ms,stddev_ms,searches_per_sec,speedup,heavy_count,light_count").unwrap();
    
    for r in results {
        let times: Vec<f64> = r.samples.iter().map(|s| s.total_time_ms).collect();
        let stddev = stddev_of(&times);
        let (heavy, light) = if !r.samples.is_empty() {
            (r.samples[0].heavy_move_count, r.samples[0].light_move_count)
        } else {
            (0, 0)
        };
        
        writeln!(file, "{},{},{},{},{},{:.2},{:.3},{:.3},{:.3},{:.3},{:.3},{:.2},{:.3},{},{}",
            ts, depth, r.position_name, r.config_name, r.probe_depth, r.heavy_ratio,
            r.median_total_ms, r.median_probe_ms, r.median_phase1_ms, r.median_phase2_ms,
            stddev, r.searches_per_second, r.speedup_vs_baseline, heavy, light
        ).unwrap();
    }
    
    println!("\nResults exported to: {}", path);
}

fn print_two_phase_summary(results: &[TwoPhaseBenchmarkResult]) {
    println!("\n=== TWO-PHASE BENCHMARK SUMMARY ===\n");
    println!("{:<12} {:<20} {:>12} {:>10} {:>10} {:>10} {:>8}",
        "Position", "Config", "Median(ms)", "Probe(ms)", "P1(ms)", "P2(ms)", "Speedup");
    println!("{}", "-".repeat(85));
    
    for r in results {
        println!("{:<12} {:<20} {:>12.1} {:>10.1} {:>10.1} {:>10.1} {:>7.2}x",
            r.position_name, r.config_name, r.median_total_ms,
            r.median_probe_ms, r.median_phase1_ms, r.median_phase2_ms,
            r.speedup_vs_baseline);
    }
    
    // Find best config per position
    let positions: Vec<_> = results.iter().map(|r| r.position_name.clone()).collect::<std::collections::HashSet<_>>().into_iter().collect();
    
    println!("\n=== BEST CONFIGURATIONS ===\n");
    for pos in &positions {
        let best = results.iter()
            .filter(|r| &r.position_name == pos)
            .max_by(|a, b| a.speedup_vs_baseline.partial_cmp(&b.speedup_vs_baseline).unwrap());
        
        if let Some(b) = best {
            println!("{}: {} ({:.2}x speedup, probe_depth={}, heavy_ratio={:.1})",
                pos, b.config_name, b.speedup_vs_baseline, b.probe_depth, b.heavy_ratio);
        }
    }
}
