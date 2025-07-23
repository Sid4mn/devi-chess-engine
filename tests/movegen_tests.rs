use devi::moves::*;
use devi::board::*;
use devi::types::*;

#[test]
fn test_empty_move_generation() {
    let board = Board::new();
    let moves = generate_moves(&board, Color::White);
    assert_eq!(moves.len(), 0);
}

#[test]
fn test_pawn_moves_starting_position() {
    let mut board = Board::new();
    board.setup_starting_position();

    let moves = generate_pawn_moves(&board, Square(51), Color::Black);
    assert_eq!(moves.len(), 2);

    let moves2 = generate_pawn_moves(&board, Square(15), Color::White);
    assert_eq!(moves2.len(), 2);
}

#[test]
fn test_all_starting_moves() {
    let mut board = Board::new();
    board.setup_starting_position();

    let white_moves = generate_moves(&board, Color::White);
    assert_eq!(white_moves.len(), 20);

    let black_moves = generate_moves(&board, Color::Black);
    assert_eq!(black_moves.len(), 20);
}

#[test]
fn test_pawn_captures() {
    let mut board = Board::new();

    board.set_piece(Square(28), Some(Piece { piece_type: PieceType::Pawn, color: Color::White })); //e4

    board.set_piece(Square(35), Some(Piece { piece_type: PieceType::Pawn, color: Color::Black })); //d5
    board.set_piece(Square(37), Some(Piece { piece_type: PieceType::Pawn, color: Color::Black })); //f5

    let moves = generate_pawn_moves(&board, Square(28), Color::White);

    assert_eq!(moves.len(), 3);

    let capture_count = moves.iter()
            .filter(|matched| matched.to.0 == 35 || matched.to.0 == 37)
            .count();

    assert_eq!(capture_count, 2);
}

#[test]
fn test_en_passant_capture() {
    let mut board = Board::new();

    board.set_piece(Square(35), Some(Piece { piece_type: PieceType::Pawn, color: Color::White }));

    board.en_passant = Some(Square(42));

    let moves = generate_pawn_moves(&board, Square(35), Color::White);

    let count = moves.iter()
            .filter(|matched| matched.special_move == Some(SpecialMove::EnPassant))
            .count();

    assert_eq!(count, 1);
}

#[test]
fn test_pawn_captures_boundaries() {
    let mut board = Board::new();

    board.set_piece(Square(24), Some(Piece { piece_type: PieceType::Pawn, color: Color::White }));
    board.set_piece(Square(33), Some(Piece { piece_type: PieceType::Pawn, color: Color::Black }));

    board.set_piece(Square(31), Some(Piece { piece_type: PieceType::Pawn, color: Color::White }));
    board.set_piece(Square(38), Some(Piece { piece_type: PieceType::Pawn, color: Color::Black }));

    let moves_24 = generate_pawn_moves(&board, Square(24), Color::White);
    assert_eq!(moves_24.len(), 2);

    let moves_31 = generate_pawn_moves(&board, Square(31), Color::White);
    assert_eq!(moves_31.len(), 2);
}

#[test]
fn test_knight_moves_center() {
    let mut board = Board::new();
    
    board.set_piece(Square(27), Some(Piece { piece_type: PieceType::Knight, color: Color::White }));
    
    let moves = generate_knight_moves(&board, Square(27), Color::White);
    
    //Knight in center should have 8 possible moves
    assert_eq!(moves.len(), 8);
}

#[test]
fn test_knight_moves_corner() {
    let mut board = Board::new();
    
    //Place knight on a1 (corner)
    board.set_piece(Square(0), Some(Piece { piece_type: PieceType::Knight, color: Color::White }));
    
    let moves = generate_knight_moves(&board, Square(0), Color::White);
    
    //Knight in corner should have only 2 possible moves
    assert_eq!(moves.len(), 2);
}

#[test]
fn test_knight_captures() {
    let mut board = Board::new();
    
    //Place white knight on d4
    board.set_piece(Square(27), Some(Piece { piece_type: PieceType::Knight, color: Color::White }));
    
    // Place black pieces on some knight target squares
    board.set_piece(Square(44), Some(Piece { piece_type: PieceType::Pawn, color: Color::Black })); // c6
    board.set_piece(Square(46), Some(Piece { piece_type: PieceType::Pawn, color: Color::Black })); // e6
    
    let moves = generate_knight_moves(&board, Square(27), Color::White);
    
    //Should still have 8 moves
    assert_eq!(moves.len(), 8);
}

#[test]
fn test_king_moves_center() {
    let mut board = Board::new();
    
    //Place king on d4
    board.set_piece(Square(27), Some(Piece { piece_type: PieceType::King, color: Color::White }));
    
    let moves = generate_king_moves(&board, Square(27), Color::White);
    
    //King should have 8 possible moves (all adjacent squares)
    assert_eq!(moves.len(), 8);
}

#[test]
fn test_king_moves_edge() {
    let mut board = Board::new();
    
    //Place king on a4 edge
    board.set_piece(Square(24), Some(Piece { piece_type: PieceType::King, color: Color::White }));
    
    let moves = generate_king_moves(&board, Square(24), Color::White);
    
    //King on edge should have 5 possible moves
    assert_eq!(moves.len(), 5);
}

#[test]
fn test_king_blocked_by_own_pieces() {
    let mut board = Board::new();
    
    //Place white king on d4
    board.set_piece(Square(27), Some(Piece { piece_type: PieceType::King, color: Color::White }));
    
    //Block some squares with white pieces
    board.set_piece(Square(34), Some(Piece { piece_type: PieceType::Pawn, color: Color::White })); //c5
    board.set_piece(Square(35), Some(Piece { piece_type: PieceType::Pawn, color: Color::White })); //d5
    
    let moves = generate_king_moves(&board, Square(27), Color::White);
    
    //Should have 6 moves
    assert_eq!(moves.len(), 6);
}

#[test]
fn test_rook_moves_center() {
    let mut board = Board::new();
    
    //Place rook on d4 (center)
    board.set_piece(Square(27), Some(Piece { piece_type: PieceType::Rook, color: Color::White }));
    
    let moves = generate_rook_moves(&board, Square(27), Color::White);
    
    //Rook in center should have 14 moves (7 horizontal + 7 vertical)
    assert_eq!(moves.len(), 14);
}

#[test]
fn test_rook_moves_corner() {
    let mut board = Board::new();
    
    //Place rook on a1 (corner)
    board.set_piece(Square(0), Some(Piece { piece_type: PieceType::Rook, color: Color::White }));
    
    let moves = generate_rook_moves(&board, Square(0), Color::White);

    println!("Rook corner moves: {}", moves.len()); // Debug line
    for mv in &moves {
        println!("Move: {} -> {}", mv.from.0, mv.to.0);
    }
    
    //Rook in corner should have 14 moves (7 up + 7 right)
    assert_eq!(moves.len(), 14);
}

#[test]
fn test_rook_blocked_by_pieces() {
    let mut board = Board::new();
    
    //Place white rook on d4
    board.set_piece(Square(27), Some(Piece { piece_type: PieceType::Rook, color: Color::White }));
    
    //Block with white piece (can't capture)
    board.set_piece(Square(35), Some(Piece { piece_type: PieceType::Pawn, color: Color::White })); //d5
    
    //Block with black piece (can capture)
    board.set_piece(Square(19), Some(Piece { piece_type: PieceType::Pawn, color: Color::Black })); //d3
    
    let moves = generate_rook_moves(&board, Square(27), Color::White);
    
    //Should have fewer moves due to blocking
    assert!(moves.len() < 14);
    
    //Should have a capture move to d3
    let capture_moves = moves.iter().filter(|m| m.to.0 == 19).count();
    assert_eq!(capture_moves, 1);
}

#[test]
fn test_bishop_moves_center() {
    let mut board = Board::new();
    
    //Place bishop on d4 (center)
    board.set_piece(Square(27), Some(Piece { piece_type: PieceType::Bishop, color: Color::White }));
    
    let moves = generate_bishop_moves(&board, Square(27), Color::White);
    
    //Bishop in center should have 13 moves (all diagonal directions)
    assert_eq!(moves.len(), 13);
}

#[test]
fn test_bishop_moves_corner() {
    let mut board = Board::new();
    
    //Place bishop on a1 (corner)
    board.set_piece(Square(0), Some(Piece { piece_type: PieceType::Bishop, color: Color::White }));
    
    let moves = generate_bishop_moves(&board, Square(0), Color::White);
    
    //Bishop in corner should have 7 moves (one diagonal)
    assert_eq!(moves.len(), 7);
}

#[test]
fn test_bishop_captures() {
    let mut board = Board::new();
    
    //Place white bishop on d4
    board.set_piece(Square(27), Some(Piece { piece_type: PieceType::Bishop, color: Color::White }));
    
    //Place black pieces on diagonals
    board.set_piece(Square(36), Some(Piece { piece_type: PieceType::Pawn, color: Color::Black })); // c5
    board.set_piece(Square(18), Some(Piece { piece_type: PieceType::Pawn, color: Color::Black })); // c3
    
    let moves = generate_bishop_moves(&board, Square(27), Color::White);
    
    //Should include captures
    let capture_count = moves.iter()
        .filter(|m| m.to.0 == 36 || m.to.0 == 18)
        .count();
    assert_eq!(capture_count, 2);
}

#[test]
fn test_queen_moves_center() {
    let mut board = Board::new();
    
    //Place queen on d4 (center)
    board.set_piece(Square(27), Some(Piece { piece_type: PieceType::Queen, color: Color::White }));
    
    let moves = generate_queen_moves(&board, Square(27), Color::White);
    
    //Queen should have 27 moves (14 rook + 13 bishop)
    assert_eq!(moves.len(), 27);
}

#[test]
fn test_queen_moves_corner() {
    let mut board = Board::new();
    
    //Place queen on a1 (corner)
    board.set_piece(Square(0), Some(Piece { piece_type: PieceType::Queen, color: Color::White }));
    
    let moves = generate_queen_moves(&board, Square(0), Color::White);
    
    //Queen in corner should have 21 moves (14 rook + 7 bishop)
    assert_eq!(moves.len(), 21);
}

#[test]
fn test_queen_captures_and_blocks() {
    let mut board = Board::new();
    
    //Place white queen on d4
    board.set_piece(Square(27), Some(Piece { piece_type: PieceType::Queen, color: Color::White }));
    
    board.set_piece(Square(35), Some(Piece { piece_type: PieceType::Pawn, color: Color::White })); //d5 - blocks
    board.set_piece(Square(36), Some(Piece { piece_type: PieceType::Pawn, color: Color::Black })); //c5 - capture
    
    let moves = generate_queen_moves(&board, Square(27), Color::White);
    
    assert!(moves.len() < 27);
    
    let captures = moves.iter().filter(|m| m.to.0 == 36).count();
    assert_eq!(captures, 1);
}