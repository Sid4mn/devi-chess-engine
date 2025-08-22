use crate::board::{Board, BoardRepresentation};
use crate::moves::piece_moves::*;
use crate::types::*;

fn generate_castling_moves(board: &Board, color: Color) -> Vec<Move> {
    let mut moves = Vec::new();

    if board.is_in_check(color) {
        return moves; // Can't castle out of check
    }

    let (king_square, kingside_right, queenside_right) = match color {
        Color::White => (Square(4), WK, WQ),
        Color::Black => (Square(60), BK, BQ),
    };

    let opponent_color = match color {
        Color::White => Color::Black,
        Color::Black => Color::White,
    };

    let castling_rights = board.castling_rights();

    // Kingside castling
    if (castling_rights & kingside_right) != 0 {
        let f_square = Square(king_square.0 + 1);
        let g_square = Square(king_square.0 + 2);
        let h_square = Square(king_square.0 + 3);

        if board.is_empty(f_square) && board.is_empty(g_square) {
            // Check if rook is on h-file
            if let Some(piece) = board.get_piece(h_square) {
                if piece.piece_type == PieceType::Rook && piece.color == color {
                    if !board.is_square_attacked(king_square, opponent_color)
                        && !board.is_square_attacked(f_square, opponent_color)
                        && !board.is_square_attacked(g_square, opponent_color)
                    {
                        moves.push(Move {
                            from: king_square,
                            to: g_square,
                            special_move: Some(SpecialMove::Castle),
                            promotion: None,
                        });
                    }
                }
            }
        }
    }

    // Queenside castling
    if (castling_rights & queenside_right) != 0 {
        let d_square = Square(king_square.0 - 1);
        let c_square = Square(king_square.0 - 2);
        let b_square = Square(king_square.0 - 3);
        let a_square = Square(king_square.0 - 4);

        if board.is_empty(d_square) && board.is_empty(c_square) && board.is_empty(b_square) {
            // Check if rook is on a-file
            if let Some(piece) = board.get_piece(a_square) {
                if piece.piece_type == PieceType::Rook && piece.color == color {
                    if !board.is_square_attacked(king_square, opponent_color)
                        && !board.is_square_attacked(c_square, opponent_color)
                        && !board.is_square_attacked(d_square, opponent_color)
                    {
                        moves.push(Move {
                            from: king_square,
                            to: c_square,
                            special_move: Some(SpecialMove::Castle),
                            promotion: None,
                        });
                    }
                }
            }
        }
    }

    moves
}

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

    moves.extend(generate_castling_moves(board, color));

    moves
}
