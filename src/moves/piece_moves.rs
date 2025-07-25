use crate::types::*;
use crate::board::{Board, BoardRepresentation};

const KNIGHT_DIRS: [i8; 8] = [17, 15, 10, 6, -6, -10, -15, -17];
const KING_DIRS: [i8; 8] = [7, 8, 9, -1, 1, -7, -8, -9];
const ROOK_DIRS: [i8; 4] = [8, -8, 1, -1];
const BISHOP_DIRS: [i8; 4] = [9, -9, 7, -7];

// PAWN MOVES ================================
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

    if let Some(en_passant_square) = board.en_passant() {
        let square_idx = square.0 as i8;
        let en_passant_idx: i8 = en_passant_square.0 as i8;
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

// KNIGHT MOVES ================================
pub fn generate_knight_moves(board: &Board, square: Square, color: Color) -> Vec<Move> {
    let mut moves = Vec::new();

    let square_idx = square.0 as i8;

    let file = square_idx % 8;
    let rank = square_idx / 8;

    for &dir in KNIGHT_DIRS.iter() {
        let target_idx = square_idx + dir;
        
        if target_idx < 0 || target_idx >= 64 {
            continue;
        } 

        let target_file = target_idx % 8;
        let target_rank = target_idx / 8;
        
        // if (file - target_file).abs() > 2 || (rank - target_rank).abs() > 2 {
        //     continue;
        // }
        //handles file wrap check (knights should not be able to wrap around board edges)

        let file_diff = (file - target_file).abs();
        let rank_diff = (rank - target_rank).abs();
        if !((file_diff == 2 && rank_diff == 1) || (file_diff == 1 && rank_diff == 2)) {
            continue;
        }

        let target_square = Square(target_idx as u8);

       //check target_square, if enemy/empty
       match board.get_piece(target_square) {
        None => {
            moves.push(Move { 
                from: square, 
                to: target_square, 
                special_move: None 
            });
        }
        Some(piece) if piece.color != color => {
            moves.push(Move {
                from: square,
                to: target_square,
                special_move: None,
            });
        }
        Some(_ally_piece) => {
            continue;
        }
       }

    }
    moves
}

// KING MOVES ================================
pub fn generate_king_moves(board: &Board, square: Square, color: Color) -> Vec<Move> {
    let mut moves = Vec::new();
    let square_idx = square.0 as i8;
    let file = square_idx % 8;
    let rank = square_idx / 8;

    for &dir in KING_DIRS.iter() {
        let target_idx = square_idx + dir;
        if target_idx < 0 || target_idx >= 64 {
            continue;
        }

        let target_file = target_idx % 8;
        let target_rank = target_idx / 8;

        //handle king movement wrapping around the board
        if (file - target_file).abs() > 1 || (rank - target_rank).abs() > 1 {
            continue;
        }

        let target_square = Square(target_idx as u8);

        match board.get_piece(target_square) {
        None => {
            moves.push(Move { 
                from: square, 
                to: target_square, 
                special_move: None 
            });
        }
        Some(piece) if piece.color != color => {
            moves.push(Move {
                from: square,
                to: target_square,
                special_move: None,
            });
        }
        Some(_ally_piece) => {
            continue;
        }
       }
    }
    moves
}

// ROOK MOVES ================================
pub fn generate_rook_moves(board: &Board, square: Square, color: Color) -> Vec<Move> {
    let mut moves = Vec::new();
    let square_idx = square.0 as i8; 
    let file = square_idx % 8;

    for &dir in ROOK_DIRS.iter() {
        let mut step = 1;

        loop {
            let target_idx = square_idx + (dir * step);

            if target_idx < 0 || target_idx >= 64 {
                break;
            }
            let target_file = target_idx % 8;

            if dir == 1 && target_file <= file {
                break;
            }
            if dir == -1 && target_file >= file {
                break;
            }

            let target_square = Square(target_idx as u8);

            match board.get_piece(target_square) {
                None => {
                    moves.push(Move { 
                        from: square, 
                        to: target_square, 
                        special_move: None 
                    });
                }
                Some(piece) if piece.color != color => {
                    moves.push(Move { 
                        from: square, 
                        to: target_square, 
                        special_move: None 
                    });
                    break;
                }
                Some(_ally_piece) => {
                    break;
                }
            }

            step += 1;
        }
    }


    moves
}

// BISHOP MOVES ================================
pub fn generate_bishop_moves(board: &Board, square: Square, color: Color) -> Vec<Move> {
    let mut moves = Vec::new();
    let square_idx = square.0 as i8; 
    
    let file = square_idx % 8;
    let rank = square_idx / 8;

    for &dir in BISHOP_DIRS.iter() {
        let mut step = 1;

        loop {
            let target_idx = square_idx + (dir * step);

            if target_idx < 0 || target_idx >= 64 {
                break;
            }
            let target_file = target_idx % 8;
            let target_rank = target_idx / 8;

            if (file - target_file).abs() != (rank - target_rank).abs() {
                break;
            }

            let target_square = Square(target_idx as u8);

            match board.get_piece(target_square) {
                None => {
                    moves.push(Move { 
                        from: square, 
                        to: target_square, 
                        special_move: None 
                    });
                }
                Some(piece) if piece.color != color => {
                    moves.push(Move { 
                        from: square, 
                        to: target_square, 
                        special_move: None 
                    });
                    break;
                }
                Some(_ally_piece) => {
                    break;
                }
            }

            step += 1;
        }
    }


    moves
}

// QUEEN MOVES ================================
pub fn generate_queen_moves(board: &Board, square: Square, color: Color) -> Vec<Move> {
    let mut moves = Vec::new();

    moves.extend(generate_rook_moves(board, square, color));
    moves.extend(generate_bishop_moves(board, square, color));

    moves
}