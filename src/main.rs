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
    let white_pawn = Piece::new(PieceType::Pawn, Color::White);

    board.set_piece(Square(8), Some(white_pawn));

    println!("Piece at a2: {:?}", board.get_piece(Square(8)));
    println!("e4 is empty? : {:?}", board.is_empty(Square(28)));


}
