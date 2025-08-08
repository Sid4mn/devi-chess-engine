use devi::board::{Board, BoardRepresentation};
use devi::evaluation::evaluate;
use devi::search::{search};
use devi::benchmark::{BenchmarkConfig, run_benchmark};

fn main() {
    println!("devi Chess Engine v0.1.0");
    println!("------------------------");

    let mut board = Board::new();
    board.setup_starting_position();
    
    println!("Starting position evaluation: {}", evaluate(&board));
    
    // Single search test
    println!("Searching to depth 4...");
    let (best_move, _score) = search(&mut board, 4);
    println!("Best move: {} -> {}", best_move.from.0, best_move.to.0);
    
    // Run professional benchmark
    let config = BenchmarkConfig::default();
    let results = run_benchmark(&config);
    
    // Export results for visualization
    export_benchmark_csv(&results);
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