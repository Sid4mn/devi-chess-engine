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

    let promotion_rank = match color {
        Color::White => square_idx >= 48 && square_idx <= 55,
        Color::Black => square_idx >= 8 && square_idx <= 15,
    };

    let one_forward = square_idx + direction; 
    if one_forward >= 0 && one_forward < 64 {
        let target_square = Square(one_forward as u8);
        if board.is_empty(target_square) {
            if promotion_rank {
                // Generate all 4 promotion moves
                for piece_type in [PieceType::Queen, PieceType::Rook, PieceType::Bishop, PieceType::Knight] {
                    moves.push(Move {
                        from: square,
                        to: target_square,
                        special_move: Some(SpecialMove::Promotion),
                        promotion: Some(piece_type),
                    });
                }
            } else {
                moves.push(Move {
                    from: square,
                    to: target_square,
                    special_move: None,
                    promotion: None,
                });
            }

            // Double push logic (only if not promotion rank)
            if !promotion_rank {
                let starting_rank = match color {
                    Color::White => square_idx >= 8 && square_idx <= 15,
                    Color::Black => square_idx >= 48 && square_idx <= 55,
                };

                if starting_rank {
                    let two_forward = square_idx + (direction * 2);
                    if two_forward >= 0 && two_forward < 64 {
                        let double_target = Square(two_forward as u8);
                        if board.is_empty(double_target) {
                            moves.push(Move {
                                from: square,
                                to: double_target,
                                special_move: None,
                                promotion: None,
                            });
                        }
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
    
    let promotion_rank = match color {
        Color::White => square_idx >= 48 && square_idx <= 55, // 7th rank
        Color::Black => square_idx >= 8 && square_idx <= 15,   // 2nd rank
    };

    let file = square_idx % 8;

    // Define capture offsets based on color
    let (west_offset, east_offset) = match color {
        Color::White => (7, 9),   // White captures NW (+7) and NE (+9)
        Color::Black => (-9, -7), // Black captures SW (-9) and SE (-7)
    };

    // West capture
    if file > 0 {
        let target_idx = square_idx + west_offset;
        if target_idx >= 0 && target_idx < 64 {
            let target_square = Square(target_idx as u8);
            if let Some(piece) = board.get_piece(target_square) {
                if piece.color != color {
                    if promotion_rank {
                        // Generate all 4 promotion captures
                        for piece_type in [PieceType::Queen, PieceType::Rook, PieceType::Bishop, PieceType::Knight] {
                            moves.push(Move {
                                from: square,
                                to: target_square,
                                special_move: Some(SpecialMove::Promotion),
                                promotion: Some(piece_type),
                            });
                        }
                    } else {
                        moves.push(Move {
                            from: square,
                            to: target_square,
                            special_move: None,
                            promotion: None,
                        });
                    }
                }
            }
        }
    }

    // East capture
    if file < 7 {
        let target_idx = square_idx + east_offset;
        if target_idx >= 0 && target_idx < 64 {
            let target_square = Square(target_idx as u8);
            if let Some(piece) = board.get_piece(target_square) {
                if piece.color != color {
                    if promotion_rank {
                        // Generate all 4 promotion captures
                        for piece_type in [PieceType::Queen, PieceType::Rook, PieceType::Bishop, PieceType::Knight] {
                            moves.push(Move {
                                from: square,
                                to: target_square,
                                special_move: Some(SpecialMove::Promotion),
                                promotion: Some(piece_type),
                            });
                        }
                    } else {
                        moves.push(Move {
                            from: square,
                            to: target_square,
                            special_move: None,
                            promotion: None,
                        });
                    }
                }
            }
        }
    }

    // En passant capture
    if let Some(en_passant_square) = board.en_passant() {
        let en_passant_idx = en_passant_square.0 as i8;
        let ep_file = en_passant_idx % 8;
        let rank = square_idx / 8;
        
        // Check if pawn is on correct rank for en passant
        let can_capture_ep = match color {
            Color::White => rank == 4,  // White pawns on 5th rank
            Color::Black => rank == 3,  // Black pawns on 4th rank
        };
        
        if can_capture_ep && (ep_file - file).abs() == 1 {
            moves.push(Move {
                from: square,
                to: en_passant_square,
                special_move: Some(SpecialMove::EnPassant),
                promotion: None,
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
        
        //handles file wrap check (knights should not be able to wrap around board edges)
        if (file - target_file).abs() > 2 || (rank - target_rank).abs() > 2 {
            continue;
        }

        let target_square = Square(target_idx as u8);

       //check target_square, if enemy/empty
       match board.get_piece(target_square) {
        None => {
            moves.push(Move { 
                from: square, 
                to: target_square, 
                special_move: None,
                promotion: None,
            });
        }
        Some(piece) if piece.color != color => {
            moves.push(Move {
                from: square,
                to: target_square,
                special_move: None,
                promotion: None,
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
                special_move: None,
                promotion: None,
            });
        }
        Some(piece) if piece.color != color => {
            moves.push(Move {
                from: square,
                to: target_square,
                special_move: None,
                promotion: None,
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
            if let Some(intermediate) = dir.checked_mul(step) {
                if let Some(target_idx) = square_idx.checked_add(intermediate) {

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
                            special_move: None,
                            promotion: None,
                        });
                    }
                    Some(piece) if piece.color != color => {
                        moves.push(Move { 
                            from: square, 
                            to: target_square, 
                            special_move: None,
                            promotion: None,
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
            if let Some(intermediate) = dir.checked_mul(step) {
                if let Some(target_idx) = square_idx.checked_add(intermediate) {
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
                                special_move: None,
                                promotion: None,
                            });
                        }
                        Some(piece) if piece.color != color => {
                            moves.push(Move { 
                                from: square, 
                                to: target_square, 
                                special_move: None,
                                promotion: None,
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