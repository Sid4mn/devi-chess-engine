use devi::moves::generate_legal_moves;
use devi::types::*;
use devi::board::*;


#[test]
fn test_empty_board_creation() {
    let board = Board::new();
    assert!(board.is_empty(Square(0))); //a1
    assert!(board.is_empty(Square(20))); //e3
    assert!(board.is_empty(Square(63))); //h8
    assert!(board.is_empty(Square(35))); //d4
}

#[test]
fn test_piece_placement_and_retrieval() {
    let mut board = Board::new();

    let white_piece = Piece::new(PieceType::Pawn, Color::White);
    let black_piece = Piece::new(PieceType::King, Color::Black);

    board.set_piece(Square(23), Some(white_piece));
    board.set_piece(Square(60), Some(black_piece));

    let retrieved_white = board.get_piece(Square(23));
    let retrieved_black = board.get_piece(Square(60));

    assert_eq!(retrieved_white, Some(white_piece));
    assert_eq!(retrieved_black, Some(black_piece));

    assert!(board.is_empty(Square(39))); //h5
}

#[test]
fn test_starting_position_setup() {
    let mut board = Board::new();
    board.setup_starting_position();
    
    // Test white pieces
    assert_eq!(board.get_piece(Square(4)), Some(Piece::new(PieceType::King, Color::White))); // e1
    assert_eq!(board.get_piece(Square(3)), Some(Piece::new(PieceType::Queen, Color::White))); // d1
    assert_eq!(board.get_piece(Square(0)), Some(Piece::new(PieceType::Rook, Color::White))); // a1
    
    // Test black pieces  
    assert_eq!(board.get_piece(Square(60)), Some(Piece::new(PieceType::King, Color::Black))); // e8
    assert_eq!(board.get_piece(Square(59)), Some(Piece::new(PieceType::Queen, Color::Black))); // d8
    
    // Test pawns
    assert_eq!(board.get_piece(Square(8)), Some(Piece::new(PieceType::Pawn, Color::White))); // a2
    assert_eq!(board.get_piece(Square(48)), Some(Piece::new(PieceType::Pawn, Color::Black))); // a7
    
    // Test empty center
    assert!(board.is_empty(Square(28))); // e4
    assert!(board.is_empty(Square(35))); // d5

    let black_legal_moves = generate_legal_moves(&mut board, Color::Black);

    assert_eq!(black_legal_moves.len(), 20);

}