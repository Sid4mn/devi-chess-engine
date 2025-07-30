use devi::moves::perft::{perft_divide};
use devi::moves::{perft};
use devi::board::{Board, BoardRepresentation};

fn main() {
    println!("devi Chess Engine v0.1.0");
    println!("------------------------");

    let mut board = Board::new();
    board.setup_starting_position();


    println!("Perft(1): {}", perft(&mut board, 1));//Expected: 20
    println!("Perft(2): {}", perft(&mut board, 2));//Expected: 400
    println!("Perft(3): {}", perft(&mut board, 3));//Expected: 8902
    println!("Perft(4): {}", perft(&mut board, 4));//Expected: 197281
    // println!("Perft(5): {}", perft(&mut board, 5));//Expected: 4865609
    // println!("Perft(6): {}", perft(&mut board, 6));//Expected: 119060324

    println!("P divide (3): {}", perft_divide(&mut board, 3).1);
    println!("P divide (4): {}", perft_divide(&mut board, 4).1);

}

