use devi::board::{Board, BoardRepresentation};
use devi::moves::perft::perft_detailed_parallel;
use std::time::Instant;

fn main() {
    let mut board = Board::new();
    board.setup_starting_position();
    
    println!("Parallel Detailed Perft - Starting Position (Depth 1-8)");
    println!("========================================================");
    println!();
    println!(
        "{:>5} {:>18} {:>12} {:>8} {:>8} {:>8} {:>10} {:>8} {:>10} {:>10}",
        "Depth", "Nodes", "Captures", "E.p.", "Castles", "Promos", "Checks", "DblChk", "Mates", "Time"
    );
    println!("{:-<115}", "");
    
    for depth in 1..=8 {
        let start = Instant::now();
        let stats = perft_detailed_parallel(&mut board, depth);
        let elapsed = start.elapsed();
        
        println!(
            "{:>5} {:>18} {:>12} {:>8} {:>8} {:>8} {:>10} {:>8} {:>10} {:>10.2?}",
            depth,
            stats.nodes,
            stats.captures,
            stats.en_passant,
            stats.castles,
            stats.promotions,
            stats.checks,
            stats.double_checks,
            stats.checkmates,
            elapsed
        );
    }
    
    println!();
    println!("CPW Reference Values:");
    println!("Depth 6: nodes=119,060,324 captures=2,812,008 ep=5,248 checks=809,099 mates=10,828");
    println!("Depth 7: nodes=3,195,901,860 captures=108,329,926 ep=319,617 castles=883,453 checks=33,103,848 mates=435,767");
    println!("Depth 8: nodes=84,998,978,956 captures=3,523,740,106 ep=7,187,977 castles=23,605,205 checks=968,981,593 mates=9,852,036");
}