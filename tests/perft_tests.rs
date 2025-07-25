use devi::board::{BoardRepresentation, Board};
use devi::moves::perft;

#[test]
fn test_perft_starting_position() {
    let mut board = Board::new();
    board.setup_starting_position();

    assert_eq!(perft(&mut board, 1),20);
    assert_eq!(perft(&mut board, 2), 400);
    assert_eq!(perft(&mut board, 3), 8_902);
}
