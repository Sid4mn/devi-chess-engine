use crate::board::{Board, BoardRepresentation};
use crate::types::*;

const PIECE_VALUES: [i32; 6] = [
    100, //PAWN
    320, //KNIGHT
    330, //BISHOP
    500, //ROOK
    900, //QUEEN
    20000, //KING
];

pub fn evaluate(board: &Board) -> i32 {
    let mut score = 0;

    for square_idx in 0..64 {
        let square = Square(square_idx);
        if let Some(piece) = board.get_piece(square) {
            let piece_value = match piece.piece_type {
                PieceType::Pawn => PIECE_VALUES[0],
                PieceType::Knight => PIECE_VALUES[1],
                PieceType::Bishop => PIECE_VALUES[2],
                PieceType::Rook => PIECE_VALUES[3],
                PieceType::Queen => PIECE_VALUES[4],
                PieceType::King => PIECE_VALUES[5],
            };

            match piece.color {
                Color::White => score += piece_value,
                Color::Black => score -= piece_value,
            }
        }

    }

    score
}