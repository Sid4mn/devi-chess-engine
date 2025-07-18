use devi::moves::generate_moves;
use devi::types::*;
use devi::board::*;
// use devi::moves::*;
// use devi::utils::*;
// use devi::search::*;
// use devi::evaluation::*;

fn main() {
    println!("devi Chess Engine v0.1.0");
    println!("------------------------");

    let mut board = Board::new();    
    board.setup_starting_position();

    //starting positions
    println!("Piece at a2: {:?}", board.get_piece(Square(8)));
    println!("e4 is empty? : {:?}", board.is_empty(Square(28)));

    let white_moves = generate_moves(&board, Color::White);
    let black_moves = generate_moves(&board, Color::Black);

    println!("White has {} possible moves.", white_moves.len());
    println!("Black has {} possible moves.", black_moves.len());

}
