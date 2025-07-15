use crate::moves::generate_pawn_moves;
use crate::types::*;
use crate::board::Board;

pub fn generate_moves(board: &Board, color: Color) -> Vec<Move> {
    let mut moves = Vec::new();

    for square_idx in 0..64 {
        let square = Square(square_idx);
        if let Some(piece) = board.get_piece(square) {
            if piece.color == color && piece.piece_type == PieceType::Pawn {
                let pawn_moves = generate_pawn_moves(board, square, color);
                moves.extend(pawn_moves);
            }
        }
    }

    moves 
}