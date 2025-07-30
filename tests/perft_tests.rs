use devi::board::{BoardRepresentation, Board};
use devi::moves::perft;

#[test]
fn test_perft_starting_position() {
    let mut board = Board::new();
    board.setup_starting_position();
    
    assert_eq!(perft(&mut board, 1), 20);
    assert_eq!(perft(&mut board, 2), 400);
    assert_eq!(perft(&mut board, 3), 8_902);
    assert_eq!(perft(&mut board, 4), 197_281);
    assert_eq!(perft(&mut board, 5), 4_865_609);
}

#[test]
fn test_perft_starting_position_regression() {
    let mut board = Board::new();
    board.setup_starting_position();
    
    assert_eq!(perft(&mut board, 1), 20, "Perft(1) regression");
    assert_eq!(perft(&mut board, 2), 400, "Perft(2) regression");
    assert_eq!(perft(&mut board, 3), 8_902, "Perft(3) regression");
    assert_eq!(perft(&mut board, 4), 197_281, "Perft(4) regression");
    assert_eq!(perft(&mut board, 5), 4_865_609, "Perft(5) regression");
    
}