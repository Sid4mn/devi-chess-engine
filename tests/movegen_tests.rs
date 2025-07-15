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
fn test_all_starting_pawn_moves() {
    let mut board = Board::new();
    board.setup_starting_position();

    let white_moves = generate_moves(&board, Color::White);
    assert_eq!(white_moves.len(), 16);

    let black_moves = generate_moves(&board, Color::Black);
    assert_eq!(black_moves.len(), 16);
}