use crate::moves::piece_moves::*;
use crate::types::*;
use crate::board::{Board, BoardRepresentation};

pub fn generate_moves(board: &Board, color: Color) -> Vec<Move> {
    let mut moves = Vec::new();

    for square_idx in 0..64 {
        let square = Square(square_idx);
        if let Some(piece) = board.get_piece(square) {
            if piece.color == color {
                match piece.piece_type {
                    PieceType::Pawn => {
                        let pawn_moves = generate_pawn_moves(board, square, color);
                        moves.extend(pawn_moves);
                    }
                    PieceType::Knight => {
                        let knight_moves = generate_knight_moves(board, square, color);
                        moves.extend(knight_moves);
                    }
                    PieceType::King => {
                        let king_moves = generate_king_moves(board, square, color);
                        moves.extend(king_moves);
                    }
                    PieceType::Rook => {
                        let rook_moves = generate_rook_moves(board, square, color);
                        moves.extend(rook_moves);
                    }
                    PieceType::Bishop => {
                        let bishop_moves = generate_bishop_moves(board, square, color);
                        moves.extend(bishop_moves);
                    }
                    PieceType::Queen => {
                        let queen_moves = generate_queen_moves(board, square, color);
                        moves.extend(queen_moves);
                    }
                }
            }
        }
    }

    moves 
}