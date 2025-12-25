use crate::benchmark::{run_benchmark, BenchmarkConfig, BenchmarkResult};
use crate::board::{Board, BoardRepresentation};
use crate::cli::Cli;
use crate::evaluation::evaluate;
use crate::moves::{perft, perft_divide, perft_parallel};
use crate::scheduling::CorePolicy;
use crate::search::fault_tolerant::with_recovery;
use crate::search::parallel::parallel_search_with_fault;
use crate::search::parallel::parallel_search_with_policy;
use crate::search::{parallel_search, search, two_phase_search, TwoPhaseConfig}; 
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
        };
        
        println!("Using two-phase scheduler:");
        println!("  Probe depth: {}", config.probe_depth);
        println!("  P-cores: {}", config.p_core_threads);
        println!("  E-cores: {}", config.e_core_threads);
        
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

    let timestamp = std::time::SystemTime::now()
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
