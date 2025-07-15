use crate::types::*;
use crate::board::Board;

pub fn generate_pawn_moves(board: &Board, square: Square, color: Color) -> Vec<Move> {
    let mut moves = Vec::new();
    let square_idx = square.0 as i8;

    let direction = match color {
        Color::Black => -8,
        Color::White => 8,
    };

    let one_forward = square_idx + direction; 
    if one_forward >= 0 && one_forward < 64 {
        let target_square = Square(one_forward as u8);
        if board.is_empty(target_square) {
            moves.push(Move {
                from: square,
                to: target_square,
                special_move: None,
            });

            let starting_rank = match color {
                Color::Black => square_idx >= 48 && square_idx <= 55,
                Color::White => square_idx >= 8 && square_idx <= 15,
            };

            if starting_rank == true {
                let two_forward = one_forward + direction;
                if two_forward >= 0 && two_forward < 64 {
                    let target_square2 = Square(two_forward as u8);
                    if board.is_empty(target_square2) {
                        moves.push(Move {
                            from: square,
                            to: target_square2,
                            special_move: None,
                        });
                    }
                }
            }
        }
    }

    moves
}