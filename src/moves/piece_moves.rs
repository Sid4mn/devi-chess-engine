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
    generate_pawn_capture(board, square, color, &mut moves);

    moves
}

pub fn generate_pawn_capture(board: &Board, square: Square, color: Color, moves: &mut Vec<Move>) { 
    let square_idx = square.0 as i8;
    let direction_mul = match color {
        Color::Black => -1,
        Color::White => 1,
    };

    let file = (square.0 as i8) % 8;

    if file > 0 { // exclude A file, handle west capture

        let target_idx = square_idx + (7 * direction_mul);
        if target_idx >= 0 && target_idx < 64 {
            if let Some(piece) = board.get_piece(Square(target_idx as u8)) {
                let target_square = Square(target_idx as u8);
                if piece.color != color { // check opposite color
                    moves.push(Move {
                        from: square,
                        to: target_square,
                        special_move: None,
                    });
                }
            }
        }
    }

    if file < 7 { // exclude H file, handle east capture
        
        let target_idx = square_idx + (9 * direction_mul);
        if target_idx >= 0 && target_idx < 64 {
            if let Some(piece) = board.get_piece(Square(target_idx as u8)) {
                let target_square = Square(target_idx as u8);
                if piece.color != color {
                    moves.push(Move {
                        from: square,
                        to: target_square,
                        special_move: None,
                    });
                }
            }
        }
    }

    if let Some(en_passant_square) = board.en_passant {
        let square_idx = square.0 as i8;
        let en_passant_idx = en_passant_square.0 as i8;
        let file = square_idx % 8;
        
        let left_capture = square_idx + (7 * direction_mul);
        let right_capture = square_idx + (9 * direction_mul);

        if (left_capture == en_passant_idx && file > 0) || (right_capture == en_passant_idx && file < 7) {
            moves.push(Move {
                from : square,
                to : en_passant_square,
                special_move: Some(SpecialMove::EnPassant)
            });
        }

    }



}