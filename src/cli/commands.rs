use crate::benchmark::{run_benchmark, BenchmarkConfig, BenchmarkResult};
use crate::board::{Board, BoardRepresentation};
use crate::cli::Cli;
use crate::evaluation::evaluate;
use crate::moves::{generate_legal_moves, perft, perft_divide, perft_parallel};
use crate::scheduling::CorePolicy;
use crate::search::fault_tolerant::with_recovery;
use crate::search::parallel::parallel_search_with_fault;
use crate::search::parallel::parallel_search_with_policy;
use crate::search::parallel::{parallel_search_with_fault_set, InjectionPlan};
use crate::search::probe::{probe_move, ClassificationConfig};
use crate::search::{parallel_search, root_move_score, search, two_phase_search, two_phase_search_with_metrics, TwoPhaseConfig, TwoPhaseMetrics};
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
    faults: usize, // how many injected crashes this scenario had
}

// board to run on: --fen if given, otherwise startpos. exits cleanly on a bad fen.
fn board_from_args(args: &Cli) -> Board {
    match &args.fen {
        Some(fen) => match Board::from_fen(fen) {
            Ok(b) => {
                println!("Using position from --fen: {}", fen);
                b
            }
            Err(e) => {
                eprintln!("Invalid --fen \"{}\": {}", fen, e);
                std::process::exit(1);
            }
        },
        None => {
            let mut b = Board::new();
            b.setup_starting_position();
            b
        }
    }
}

pub fn run_full_benchmark(args: &Cli) {
    let board = board_from_args(args);
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
        fen: args.fen.clone(),
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
    let mut board = board_from_args(args);

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

        if args.threads == 1 {
            // serial search has no injection hook, so warn instead of doing nothing
            eprintln!("  Note: --inject-panic requires --threads > 1; running without injection.");
        }
        let search_fn = || {
            let mut b = board.clone();
            if args.threads == 1 {
                search(&mut b, args.depth)
            } else {
                // fault-aware variant so the panic actually fires + recovery kicks in
                parallel_search_with_fault(
                    &mut b,
                    args.depth,
                    policy,
                    args.threads,
                    mixed_ratio,
                    args.inject_panic,
                )
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

    let mut board = board_from_args(args);

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

    // must go through the fault-injecting path or the panic never fires (the old bug here).
    let board_clone = board.clone();
    let search_fn = || {
        let mut b = board_clone.clone();
        parallel_search_with_fault(&mut b, args.depth, CorePolicy::None, args.threads, 0.0, Some(5))
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

    let t1 = time1.as_secs_f64();
    if t1 > 0.0 {
        let overhead = ((time2.as_secs_f64() / t1) - 1.0) * 100.0;
        println!("\nRecovery overhead: {:.1}%", overhead);
    } else {
        println!("\nRecovery overhead: n/a (baseline completed too fast to measure)");
    }
}

pub fn run_soak_test(args: &Cli) {
    use std::time::Instant;

    println!("--- SOAK TEST ---");
    println!(
        "Threads: {}, Depth: {}, Iterations: {}",
        args.threads, args.depth, args.runs
    );

    let mut samples_ms: Vec<f64> = Vec::new();
    let base_board = board_from_args(args);

    for i in 1..=args.runs {
        let mut board = base_board.clone();

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

    let mut board = board_from_args(args);

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
        let secs = elapsed.as_secs_f64();
        let nps = if secs > 0.0 { nodes as f64 / secs } else { 0.0 };

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
    let policy = args.core_policy.unwrap_or(CorePolicy::None);
    let mixed_ratio = args.mixed_ratio;
    // honor --runs/--warmup so we can crank iterations up + pin cores when the machine is noisy.
    let iterations = args.runs;
    let warmup_per_scenario = args.warmup;

    println!("Fault Tolerance Overhead Analysis");
    println!(
        "Depth: {}, Threads: {}, Policy: {:?}, Iterations: {}, Warmup: {}\n",
        depth, threads, policy, iterations, warmup_per_scenario
    );

    let board = board_from_args(args);

    // N - we report overhead against this so the fault sweep reads as "K out of N moves".
    let n_moves = {
        let mut nb = board.clone();
        let c = nb.to_move();
        generate_legal_moves(&mut nb, c).len()
    };

    let mut results = Vec::new();

    // === Baseline ===
    println!("Scenario 1: Baseline (no recovery wrapper)");
    println!("  Warming up ({} runs)...", warmup_per_scenario);
    for _ in 0..warmup_per_scenario {
        let mut b = board.clone();
        let _ = if threads == 1 {
            search(&mut b, depth)
        } else {
            parallel_search_with_policy(&mut b, depth, policy, threads, mixed_ratio)
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
            parallel_search_with_policy(&mut b, depth, policy, threads, mixed_ratio)
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
        faults: 0,
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
                parallel_search_with_policy(&mut b, depth, policy, threads, mixed_ratio)
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
                parallel_search_with_policy(&mut b, depth, policy, threads, mixed_ratio)
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
        faults: 0,
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
                parallel_search_with_fault(&mut b, depth, policy, threads, mixed_ratio, Some(5))
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
                parallel_search_with_fault(&mut b, depth, policy, threads, mixed_ratio, Some(5))
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
        faults: 1,
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
            parallel_search_with_policy(&mut b, depth, policy, threads, mixed_ratio)
        };

        let mut b2 = board.clone();
        let (_mv2, _score2) = if threads == 1 {
            search(&mut b2, depth)
        } else {
            parallel_search_with_policy(&mut b2, depth, policy, threads, mixed_ratio)
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
        faults: 0,
    });

    /* === Fault scaling sweep ===
       inject K faults at once; per-move recovery should scale with K, not the flat ~100% of a full re-run. */
    println!("Scenario 5: Fault scaling sweep (K faults out of {} moves)", n_moves);
    let mut k = 1;
    while k <= n_moves {
        println!("  K = {} fault(s):", k);
        for _ in 0..warmup_per_scenario {
            let mut b = board.clone();
            let _ = parallel_search_with_fault_set(&mut b, depth, policy, threads, mixed_ratio, InjectionPlan::set(0..k));
        }

        let mut sweep_times = Vec::new();
        let mut sweep_move = String::new();
        let mut sweep_score = 0;
        for i in 1..=iterations {
            let mut b = board.clone();
            let start = Instant::now();
            let outcome = parallel_search_with_fault_set(&mut b, depth, policy, threads, mixed_ratio, InjectionPlan::set(0..k));
            let elapsed = start.elapsed().as_secs_f64() * 1000.0;
            sweep_times.push(elapsed);
            sweep_move = outcome.best_move.to_algebraic();
            sweep_score = outcome.score;
            println!("    Run {}: {:.3}ms", i, elapsed);
        }
        sweep_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let sweep_median = sweep_times[sweep_times.len() / 2];
        let sweep_overhead = ((sweep_median - baseline_median) / baseline_median) * 100.0;
        println!("    Median: {:.3}ms ({:+.2}%)\n", sweep_median, sweep_overhead);

        results.push(FaultMeasurement {
            scenario: "fault_sweep",
            times_ms: sweep_times,
            median_ms: sweep_median,
            best_move: sweep_move,
            score: sweep_score,
            faults: k,
        });

        k *= 2;
    }

    let csv_path = "benchmarks/fault_overhead.csv";
    export_fault_csv(&results, csv_path, depth, threads, n_moves);

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

fn export_fault_csv(results: &[FaultMeasurement], path: &str, depth: u32, threads: usize, n_moves: usize) {
    if let Some(parent) = std::path::Path::new(path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let mut file = std::fs::File::create(path).unwrap();
    let ts = chrono::Local::now().format("%Y-%m-%d_%H:%M:%S").to_string();

    // faults,n_moves are trailing so old readers of this file keep working.
    writeln!(
        file,
        "timestamp,depth,threads,scenario,median_ms,overhead_pct,move,score,min_ms,max_ms,faults,n_moves"
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
            "{},{},{},{},{:.3},{:.2},{},{},{:.3},{:.3},{},{}",
            ts,
            depth,
            threads,
            r.scenario,
            r.median_ms,
            overhead,
            r.best_move,
            r.score,
            min_ms,
            max_ms,
            r.faults,
            n_moves
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
        // bail on a bad fen instead of skipping the only position -> empty csv
        if let Err(e) = Board::from_fen(fen) {
            eprintln!("Invalid --fen \"{}\": {}", fen, e);
            std::process::exit(1);
        }
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
                probe_depth, 0.6, args.light_threshold,
                args.p_cores, args.e_cores,
                baseline_sps
            );
            all_results.push(tp_result);
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        
        // Classification ratio sweep (with best probe depth)
        for &ratio in &heavy_ratios {
            if ratio == 0.6 { continue; } // Already tested
            println!("\n[TwoPhase] probe_depth=2, ratio={:.1}", ratio);
            let tp_result = benchmark_two_phase(
                &mut board, depth, warmup, runs, pos_name,
                2, ratio, args.light_threshold,
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
    if samples.is_empty() {
        return 0.0;
    }
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

/* Classification-accuracy analysis.
   The two-phase scheduler bets that a cheap shallow PROBE (node count at depth 1..3,
   unpruned) predicts a root move's TRUE search cost at full depth. This mode tests that
   bet with no scheduler in the loop: per root move we record probe@{1,2,3} and the true
   cost = median single-thread wall-time of a full-depth search, then score how well each
   probe depth's ranking matches the true-cost ranking. */

const PROBE_DEPTHS: [u8; 3] = [1, 2, 3];
const TRUE_COST_REPEATS: usize = 3; // median over this many timed runs per move

struct MoveCostRow {
    algebraic: String,
    probe_counts: Vec<u64>, // one entry per PROBE_DEPTHS
    true_time_ms: f64,
}

pub fn run_classification_analysis(args: &Cli) {
    println!("=== CLASSIFICATION-ACCURACY ANALYSIS ===\n");
    let depth = args.depth;
    let heavy_ratio = args.heavy_ratio;
    println!(
        "True-cost depth: {}  |  probe depths: {:?}  |  heavy_ratio: {:.2}  |  timed repeats/move: {}",
        depth, PROBE_DEPTHS, heavy_ratio, TRUE_COST_REPEATS
    );
    println!("(true cost = median single-thread wall-time of a full-depth search of each root move)\n");

    let positions: Vec<(String, String)> = if let Some(ref fen) = args.fen {
        if Board::from_fen(fen).is_err() {
            eprintln!("Invalid --fen \"{}\"", fen);
            std::process::exit(1);
        }
        vec![("custom".to_string(), fen.clone())]
    } else {
        BENCHMARK_POSITIONS
            .iter()
            .map(|(n, f)| (n.to_string(), f.to_string()))
            .collect()
    };

    let csv_path = args
        .csv_output
        .as_deref()
        .unwrap_or("benchmarks/v0.5.0/classification_accuracy.csv");
    let mut csv_rows: Vec<String> = Vec::new();
    csv_rows.push(format!(
        "position,move,{},true_time_ms",
        PROBE_DEPTHS.iter().map(|d| format!("probe_d{}", d)).collect::<Vec<_>>().join(",")
    ));

    // (position, probe_depth, spearman, topk_overlap, cost_capture, k, n)
    let mut summary: Vec<(String, u8, f64, f64, f64, usize, usize)> = Vec::new();

    for (pos_name, fen) in &positions {
        let mut board = match Board::from_fen(fen) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("skip {}: {}", pos_name, e);
                continue;
            }
        };
        let color = board.to_move();
        let moves = generate_legal_moves(&mut board, color);
        let n = moves.len();
        println!("Position: {} ({} legal moves)", pos_name, n);
        println!("{}", "-".repeat(70));

        if n < 2 {
            println!("  <2 moves, nothing to classify; skipping.\n");
            continue;
        }

        // Per-move: probe counts (deterministic) + true cost (median of timed runs).
        let mut rows: Vec<MoveCostRow> = Vec::with_capacity(n);
        for mv in &moves {
            let probe_counts: Vec<u64> = PROBE_DEPTHS.iter().map(|&d| probe_move(&board, mv, d)).collect();

            let mut times: Vec<f64> = Vec::with_capacity(TRUE_COST_REPEATS);
            for _ in 0..TRUE_COST_REPEATS {
                let mut b = board.clone();
                let start = Instant::now();
                let _ = root_move_score(&mut b, mv, depth, color);
                times.push(start.elapsed().as_secs_f64() * 1000.0);
            }
            times.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let true_time_ms = times[times.len() / 2];

            rows.push(MoveCostRow { algebraic: mv.to_algebraic(), probe_counts, true_time_ms });
        }

        for r in &rows {
            csv_rows.push(format!(
                "{},{},{},{:.3}",
                pos_name,
                r.algebraic,
                r.probe_counts.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(","),
                r.true_time_ms
            ));
        }

        let true_cost: Vec<f64> = rows.iter().map(|r| r.true_time_ms).collect();
        let total_true: f64 = true_cost.iter().sum();
        let k = (((n as f32) * heavy_ratio) as usize).max(1); // matches classify_moves_with_config
        let true_heavy = top_k_indices(&true_cost, k);

        // Per-move table, sorted by true cost descending.
        let mut order: Vec<usize> = (0..n).collect();
        order.sort_by(|&a, &b| true_cost[b].partial_cmp(&true_cost[a]).unwrap());
        println!("  {:<8} {:>11} {:>11} {:>11} {:>12}", "move", "probe@1", "probe@2", "probe@3", "true(ms)");
        for &i in &order {
            let r = &rows[i];
            println!(
                "  {:<8} {:>11} {:>11} {:>11} {:>12.2}",
                r.algebraic, r.probe_counts[0], r.probe_counts[1], r.probe_counts[2], r.true_time_ms
            );
        }

        // Quality metrics per probe depth.
        println!();
        println!("  {:<10} {:>10} {:>15} {:>14}", "probe", "spearman", "top-k overlap", "cost-capture");
        for (pi, &pd) in PROBE_DEPTHS.iter().enumerate() {
            let probe_vec: Vec<f64> = rows.iter().map(|r| r.probe_counts[pi] as f64).collect();
            let sp = spearman(&probe_vec, &true_cost);
            let probe_heavy = top_k_indices(&probe_vec, k);
            let overlap = set_overlap(&probe_heavy, &true_heavy) as f64 / k as f64;
            let captured: f64 = probe_heavy.iter().map(|&i| true_cost[i]).sum();
            let cost_capture = if total_true > 0.0 { captured / total_true } else { 0.0 };
            println!("  d={:<8} {:>10.3} {:>14.0}% {:>13.0}%", pd, sp, overlap * 100.0, cost_capture * 100.0);
            summary.push((pos_name.clone(), pd, sp, overlap, cost_capture, k, n));
        }
        println!("  (top-k overlap & cost-capture at heavy_ratio={:.2} -> k={} of {})\n", heavy_ratio, k, n);
    }

    write_lines_csv(csv_path, &csv_rows);
    println!("Per-move data exported to: {}", csv_path);

    println!("\n=== SUMMARY: classification quality by probe depth ===");
    println!("spearman = ranking fidelity (1.0 perfect, 0 = noise)");
    println!("overlap  = fraction of the probe's heavy set that is truly heavy");
    println!("capture  = fraction of total real search work routed to the heavy set\n");
    println!("{:<12} {:>6} {:>10} {:>10} {:>10}", "position", "probe", "spearman", "overlap", "capture");
    for (pos, pd, sp, ov, cap, _k, _n) in &summary {
        println!("{:<12} d={:<4} {:>10.3} {:>9.0}% {:>9.0}%", pos, pd, sp, ov * 100.0, cap * 100.0);
    }
}

/* Crossover analysis. The whole two-phase bet is "keeping work off the slow E-cores helps".
   That only pays when there isn't enough parallel work for rayon to keep the 8 P-cores busy
   while the 2 E-cores quietly chew the rest. So: take one position, search just the first k of
   its root moves, and time it on a plain 10-thread pool (OS spreads over 8P+2E, work-stealing)
   vs an 8-thread P-core pool. Sweep k. Constant subtree sizes, so k is the only variable.
   No YBW seeding here on purpose - we want to isolate the scheduling, not the search. */
pub fn run_crossover_analysis(args: &Cli) {
    use crate::scheduling::create_p_core_pool;
    use rayon::prelude::*;

    println!("=== CROSSOVER: 10-core work-stealing (8P+2E) vs 8 P-cores only ===");
    let depth = args.depth;
    let runs = args.runs.max(5);
    let warmup = args.warmup.max(2);

    let mut board = board_from_args(args);
    let color = board.to_move();
    let moves = generate_legal_moves(&mut board, color);
    let n = moves.len();
    let base = board.clone();
    println!("{} legal moves | depth {} | {} runs (median)\n", n, depth, runs);

    let pool10 = rayon::ThreadPoolBuilder::new().num_threads(10).build().expect("pool10");
    let pool8 = create_p_core_pool(8).expect("pool8");

    // median wall-time to search the first k root moves on `pool`.
    let run_k = |pool: &rayon::ThreadPool, k: usize| -> f64 {
        let mvs = &moves[..k];
        let once = || {
            pool.install(|| {
                mvs.par_iter()
                    .map(|m| {
                        let mut b = base.clone();
                        root_move_score(&mut b, m, depth, color)
                    })
                    .reduce(|| i32::MIN + 1, i32::max)
            })
        };
        for _ in 0..warmup {
            let _ = once();
        }
        let mut ts = Vec::with_capacity(runs);
        for _ in 0..runs {
            let start = Instant::now();
            let _ = once();
            ts.push(start.elapsed().as_secs_f64() * 1000.0);
        }
        ts.sort_by(|a, b| a.partial_cmp(b).unwrap());
        ts[ts.len() / 2]
    };

    let mut ks: Vec<usize> = [2, 4, 6, 8, 12, 16, 24, 36, 48, 64]
        .iter()
        .cloned()
        .filter(|&k| k <= n)
        .collect();
    if ks.last() != Some(&n) {
        ks.push(n);
    }

    let mut rows = Vec::new();
    println!("{:>4} {:>12} {:>12} {:>9}", "k", "10core_ms", "8P_ms", "8P/10c");
    println!("{}", "-".repeat(44));
    for &k in &ks {
        let t10 = run_k(&pool10, k);
        let t8 = run_k(&pool8, k);
        let r = t8 / t10;
        let tag = if r < 0.98 {
            "  <- 8P (avoid E) wins"
        } else if r > 1.02 {
            "  10-core wins"
        } else {
            "  ~tie"
        };
        println!("{:>4} {:>12.3} {:>12.3} {:>9.3}{}", k, t10, t8, r, tag);
        rows.push((k, t10, t8, r));
    }

    /* include the move count in the default name so runs on different positions can't
       clobber each other (learned that the hard way - a startpos run ate the kiwipete data). */
    let default_path = format!("benchmarks/fixing_search/crossover_{}moves.csv", n);
    let path = args.csv_output.as_deref().unwrap_or(&default_path);
    if let Some(p) = std::path::Path::new(path).parent() {
        let _ = create_dir_all(p);
    }
    if let Ok(f) = File::create(path) {
        let mut w = BufWriter::new(f);
        let _ = writeln!(w, "k,tencore_ms,eightP_ms,ratio_8P_over_10c");
        for (k, t10, t8, r) in &rows {
            let _ = writeln!(w, "{},{:.3},{:.3},{:.4}", k, t10, t8, r);
        }
    }
    println!("\nwritten to {}", path);
}

fn top_k_indices(values: &[f64], k: usize) -> Vec<usize> {
    let mut idx: Vec<usize> = (0..values.len()).collect();
    idx.sort_by(|&a, &b| values[b].partial_cmp(&values[a]).unwrap()); // descending
    idx.truncate(k);
    idx
}

fn set_overlap(a: &[usize], b: &[usize]) -> usize {
    a.iter().filter(|x| b.contains(x)).count()
}

// Average-rank assignment (ties share the mean of their rank span), ascending.
fn average_ranks(values: &[f64]) -> Vec<f64> {
    let n = values.len();
    let mut idx: Vec<usize> = (0..n).collect();
    idx.sort_by(|&a, &b| values[a].partial_cmp(&values[b]).unwrap());
    let mut ranks = vec![0.0f64; n];
    let mut i = 0;
    while i < n {
        let mut j = i;
        while j + 1 < n && values[idx[j + 1]] == values[idx[i]] {
            j += 1;
        }
        let avg = (i + j) as f64 / 2.0 + 1.0; // 1-based average rank for the tie group
        for k in i..=j {
            ranks[idx[k]] = avg;
        }
        i = j + 1;
    }
    ranks
}

fn pearson(a: &[f64], b: &[f64]) -> f64 {
    let n = a.len() as f64;
    if n == 0.0 {
        return 0.0;
    }
    let ma = a.iter().sum::<f64>() / n;
    let mb = b.iter().sum::<f64>() / n;
    let (mut cov, mut va, mut vb) = (0.0, 0.0, 0.0);
    for i in 0..a.len() {
        let da = a[i] - ma;
        let db = b[i] - mb;
        cov += da * db;
        va += da * da;
        vb += db * db;
    }
    if va == 0.0 || vb == 0.0 {
        return 0.0;
    }
    cov / (va.sqrt() * vb.sqrt())
}

// Spearman = Pearson on the average ranks.
fn spearman(x: &[f64], y: &[f64]) -> f64 {
    pearson(&average_ranks(x), &average_ranks(y))
}

fn write_lines_csv(path: &str, lines: &[String]) {
    if let Some(parent) = std::path::Path::new(path).parent() {
        let _ = create_dir_all(parent);
    }
    match File::create(path) {
        Ok(f) => {
            let mut w = BufWriter::new(f);
            for line in lines {
                let _ = writeln!(w, "{}", line);
            }
        }
        Err(e) => eprintln!("Failed to write {}: {}", path, e),
    }
}
