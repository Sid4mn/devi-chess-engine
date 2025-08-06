use std::time::Instant;
use devi::moves::perft::{perft_divide};
use devi::moves::{perft};
use devi::board::{Board, BoardRepresentation};
use devi::evaluation::evaluate;
use devi::search::search;

fn main() {
    println!("devi Chess Engine v0.1.0");
    println!("------------------------");

    let mut board = Board::new();
    board.setup_starting_position();

    // println!("Perft(1): {}", perft(&mut board, 1));//Expected: 20
    // println!("Perft(2): {}", perft(&mut board, 2));//Expected: 400
    // println!("Perft(3): {}", perft(&mut board, 3));//Expected: 8902
    // println!("Perft(4): {}", perft(&mut board, 4));//Expected: 197281
    // println!("Perft(5): {}", perft(&mut board, 5));//Expected: 4865609
    // println!("Perft(6): {}", perft(&mut board, 6));//Expected: 119060324

    // println!("P divide (3): {}", perft_divide(&mut board, 3).1);
    // println!("P divide (4): {}", perft_divide(&mut board, 4).1);

    println!("Starting position evaluation: {}", evaluate(&board));
    
    // Search test
    println!("Searching to depth 4...");
    let (best_move, _score) = search(&mut board, 4);
    println!("Best move: {} -> {}", best_move.from.0, best_move.to.0);

    benchmark_search();

}

fn benchmark_search() {
    println!("\n === BENCHMARK SEARCH ===");
    let mut board = Board::new();
    board.setup_starting_position();

    let _ = search(&mut board, 2);

    let mut total_time = 0u128;
    let depth = 4;
    let runs = 5;

    for run in 1..=runs {
        board.setup_starting_position();
        let start_time = Instant::now();
        let (best_move, score) = search(&mut board, depth);
        let duration = start_time.elapsed();

        total_time += duration.as_millis();

        println!("Run {}: {:?} - Move: {} -> {}", run, duration, best_move.from.0, best_move.to.0.to_ascii_lowercase());
    }

    let avg_time_ms = total_time / runs as u128;
    let searches_per_sec = if avg_time_ms > 0 { 1000.0 / avg_time_ms as f64 } else { 0.0 };

    //calc stats
    println!("\n --- RESULTS ---");
    println!("\n Average time: {}", avg_time_ms);
    println!("\n Searches/second: {:.2}", searches_per_sec);

}
