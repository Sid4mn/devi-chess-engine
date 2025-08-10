use devi::board::{Board, BoardRepresentation};
use devi::evaluation::evaluate;
use devi::search::{parallel_search, search};
use devi::benchmark::{BenchmarkConfig, run_benchmark};
use devi::cli::{self, Cli};

fn main() {
    println!("devi Chess Engine v0.1.0");
    println!("------------------------");

    let args = cli::parse_args();

    match (args.benchmark, args.soak) {
        (true, _) => run_full_benchmark(&args),
        (_, true) => run_soak_test(&args),
        _ => run_single_search(&args),
    }
}

fn run_full_benchmark(args: &Cli) {
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

fn run_single_search(args: &Cli) {
    let mut board = Board::new();
    board.setup_starting_position();

    println!("Starting position evaluation: {}", evaluate(&board));
    println!("Searching to depth {}...", args.depth);

    let (best_move, _score) = if args.threads == 1 {
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

fn run_soak_test(args: &Cli) {
    use std::time::Instant;

    println!("--- SOAK TEST ---");
    println!("Threads: {}, Depth: {}, Iterations: {}", args.threads, args.depth, args.runs);

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
        pool.install(||parallel_search(&mut board, args.depth))
        };

        let duration_ms = start.elapsed().as_micros() as f64 / 1000.0;
        samples_ms.push(duration_ms);
        println!("Run {:3}: {:.3}ms", i, duration_ms);
        }
        //calculating stats.
        samples_ms.sort_by(|a,b| a.partial_cmp(b).unwrap());
        if samples_ms.len() > 0 {
        let len = samples_ms.len();
        let min = samples_ms[0];
        let max = samples_ms[len - 1];
        let median = if len % 2 == 0 {
            (samples_ms[len/2-1] + samples_ms[len/2]) / 2.0
        } else {
            samples_ms[len/2]
        };

        let p95_idx = ((len as f64 * 0.95) as usize).min(len - 1);
        let p95 = samples_ms[p95_idx];
        println!("Summary: min {:.3}ms, median {:.3}ms, p95 {:.3}ms, max {:.3}ms",min, median, p95, max);
        } else {
            println!("No samples collected!");
        }
}

fn export_benchmark_csv(results: &[devi::benchmark::BenchmarkResult]) {
    use std::fs::File;
    use std::io::Write;
    
    let csv_path = "benchmarks/speedup.csv";
    let mut file = File::create(csv_path).expect("Unable to create CSV file");
    
    writeln!(file, "threads,median_ms,searches_per_sec,speedup,efficiency").unwrap();
    for result in results {
        writeln!(file, "{},{:.3},{:.2},{:.2},{:.1}",
                 result.thread_count,
                 result.stats.median,
                 result.searches_per_second,
                 result.speedup,
                 result.efficiency).unwrap();
    }
    
    println!("\nBenchmark results exported to {}", csv_path);
}