use devi::board::{Board, BoardRepresentation};
use devi::evaluation::evaluate;
use devi::moves::perft;
use devi::search::search;

#[test]
fn test_perft_regression_gate() {
    let mut board = Board::new();
    board.setup_starting_position();

    assert_eq!(perft(&mut board, 1), 20, "Perft(1) regression detected!");
    assert_eq!(perft(&mut board, 2), 400, "Perft(2) regression detected!");
    assert_eq!(perft(&mut board, 3), 8_902, "Perft(3) regression detected!");
    assert_eq!(
        perft(&mut board, 4),
        197_281,
        "Perft(4) regression detected!"
    );
}

#[test]
fn test_search_smoke_test() {
    let mut board = Board::new();
    board.setup_starting_position();

    let (best_move, score) = search(&mut board, 4);

    assert!(
        score.abs() < 200,
        "Starting position score too extreme: {}",
        score
    );

    assert!(best_move.from.0 < 64, "Invalid from square");
    assert!(best_move.to.0 < 64, "Invalid to square");
}

#[test]
fn test_evaluation_starting_position() {
    let mut board = Board::new();
    board.setup_starting_position();

    assert_eq!(
        evaluate(&board),
        0,
        "Starting position should have equal material"
    );
}
