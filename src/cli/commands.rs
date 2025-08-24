use crate::benchmark::{run_benchmark, BenchmarkConfig, BenchmarkResult};
use crate::board::{Board, BoardRepresentation};
use crate::cli::Cli;
use crate::evaluation::evaluate;
use crate::moves::{perft, perft_divide, perft_parallel};
use crate::search::{parallel_search, search, search_root_fault};
use rayon;
use std::fs::{create_dir_all, File};
use std::io::{BufWriter, Write};
use std::time::Instant;

pub fn run_full_benchmark(args: &Cli) {
    let mut board = Board::new();
    board.setup_starting_position();
    println!("Starting position evaluation: {}", evaluate(&board));

    let config = BenchmarkConfig {
        depth: args.depth,
        warmup_runs: args.warmup,
        measurement_runs: args.runs,
        thread_counts: vec![1, 2, 4, 8],
    };

    let results = run_benchmark(&config);
    export_benchmark_csv(&results);
}

pub fn run_single_search(args: &Cli) {
    let mut board = Board::new();
    board.setup_starting_position();

    println!("Starting position evaluation: {}", evaluate(&board));
    println!("Searching to depth {}...", args.depth);

    let use_fault_tolerant = args.inject_panic.is_some() || args.dump_crashes;

    let (best_move, _score) = if use_fault_tolerant{
        println!("Using fault-tolerant search with panic injection at move {:?}", args.inject_panic);
        
        if args.threads == 1 {
            eprintln!("Warning: Fault injection requires multiple threads. Setting threads to 4.");
        }
        
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(if args.threads == 1 { 4 } else { args.threads })
            .build()
            .expect("Failed to create thread pool");
            
        pool.install(|| search_root_fault(&mut board, args.depth, args.inject_panic))
    } else if args.threads == 1 {
        search(&mut board, args.depth)
    } else {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(args.threads)
            .build()
            .expect("Failed to create thread pool");
        pool.install(|| parallel_search(&mut board, args.depth))
    };

    println!("Best move: {} -> {}", best_move.from.0, best_move.to.0);
}

pub fn run_fault_analysis(args: &Cli) {
    println!("--- FAULT TOLERANCE ANALYSIS ---");
    println!("Testing panic recovery overhead...");

    let mut board = Board::new();
    board.setup_starting_position();

    create_dir_all("docs").expect("Failed to create docs director");

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(args.threads)
        .build()
        .expect("Failed to create thread pool");

    let start = Instant::now();
    let (_, baseline_score) = pool.install(|| parallel_search(&mut board, args.depth));
    let baseline_time = start.elapsed();

    let test_positions = vec![0, 5, 10, 15];
    let mut fault_results = Vec::new();

        for panic_at in test_positions {
        let start = Instant::now();
        let (_mv, score) = pool.install(|| search_root_fault(&mut board, args.depth, Some(panic_at)));
        let fault_time = start.elapsed();
        
        let overhead_percent = ((fault_time.as_micros() as f64 / baseline_time.as_micros() as f64) - 1.0) * 100.0;
        
        fault_results.push((panic_at, score, fault_time, overhead_percent));
        
        println!(
            "Fault at move {}: Score={}, Time={:.3}ms, Overhead={:.1}%",
            panic_at,
            score,
            fault_time.as_micros() as f64 / 1000.0,
            overhead_percent
        );
    }
    
    // Write results to JSON
    let mut file = File::create("docs/fault_analysis.json").expect("Failed to create JSON file");
    writeln!(file, "{{").unwrap();
    writeln!(file, "  \"baseline\": {{").unwrap();
    writeln!(file, "    \"score\": {},", baseline_score).unwrap();
    writeln!(file, "    \"time_ms\": {:.3}", baseline_time.as_micros() as f64 / 1000.0).unwrap();
    writeln!(file, "  }},").unwrap();
    writeln!(file, "  \"fault_tests\": [").unwrap();
    
    for (i, (pos, score, time, overhead)) in fault_results.iter().enumerate() {
        writeln!(file, "    {{").unwrap();
        writeln!(file, "      \"fault_position\": {},", pos).unwrap();
        writeln!(file, "      \"score\": {},", score).unwrap();
        writeln!(file, "      \"time_ms\": {:.3},", time.as_micros() as f64 / 1000.0).unwrap();
        writeln!(file, "      \"overhead_percent\": {:.1}", overhead).unwrap();
        write!(file, "    }}").unwrap();
        if i < fault_results.len() - 1 {
            writeln!(file, ",").unwrap();
        } else {
            writeln!(file).unwrap();
        }
    }
    
    writeln!(file, "  ]").unwrap();
    writeln!(file, "}}").unwrap();
    
    println!("\nFault analysis results written to docs/fault_analysis.json");

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

pub fn export_benchmark_csv(results: &[BenchmarkResult]) {
    use std::fs::File;
    use std::io::Write;

    let csv_path = "benchmarks/speedup.csv";
    let mut file = File::create(csv_path).expect("Unable to create CSV file");

    writeln!(
        file,
        "threads,median_ms,searches_per_sec,speedup,efficiency"
    )
    .unwrap();
    for result in results {
        writeln!(
            file,
            "{},{:.3},{:.2},{:.2},{:.1}",
            result.thread_count,
            result.stats.median,
            result.searches_per_second,
            result.speedup,
            result.efficiency
        )
        .unwrap();
    }

    println!("\nBenchmark results exported to {}", csv_path);
}
