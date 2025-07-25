use devi::moves::{generate_legal_moves, generate_moves, perft};
use devi::types::*;
use devi::board::{Board, BoardRepresentation};

fn square_to_string(square: Square) -> String {
    let file = (square.0 % 8) as u8 + b'a';
    let rank = (square.0 / 8) + 1;
    format!("{}{}", file as char, rank)
}

fn main() {
    println!("devi Chess Engine v0.1.0");
    println!("------------------------");

    let mut board = Board::new();
    board.setup_starting_position();


    println!("Perft(1): {}", perft(&mut board, 1));//Expected: 20
    println!("Perft(2): {}", perft(&mut board, 2));//Expected: 400
    println!("Perft(3): {}", perft(&mut board, 3));//Expected: 8902
    // println!("Perft(4): {}", perft(&mut board, 4));//Expected: 197281
    // println!("Perft(5): {}", perft(&mut board, 5));//Expected: 4865609


//     let white_legal_moves = generate_legal_moves(&mut board, Color::White);    

//     for mv in &white_legal_moves {
//     if mv.special_move.is_some() {
//         println!("Special move: {:?}", mv.special_move);
//     }
// }
//     println!("White has {} legal moves.", white_legal_moves.len());

//     //starting positions
//     println!("Piece at a2: {:?}", board.get_piece(Square(8)));
//     println!("e4 is empty? : {:?}", board.is_empty(Square(28)));

//     let white_moves = generate_moves(&board, Color::White);
//     let black_moves = generate_moves(&board, Color::Black);

//     println!("White has {} possible moves.", white_moves.len());
//     println!("Black has {} possible moves.", black_moves.len());



}
