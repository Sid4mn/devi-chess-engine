use devi::board::{Board, BoardRepresentation};
use devi::moves::perft;
use devi::types::*;

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

// #[test]
// fn test_starting_position_all_depths() {
//     let mut board = Board::new();
//     board.setup_starting_position();

//     assert_eq!(perft(&mut board, 1), 20, "Perft(1) regression");
//     assert_eq!(perft(&mut board, 2), 400, "Perft(2) regression");
//     assert_eq!(perft(&mut board, 3), 8_902, "Perft(3) regression");
//     assert_eq!(perft(&mut board, 4), 197_281, "Perft(4) regression");

//     #[cfg(not(debug_assertions))]
//     {
//         assert_eq!(perft(&mut board, 5), 4_865_609, "Perft(5) regression");
//         assert_eq!(perft(&mut board, 6), 119_060_324, "Perft(6) regression");
//     }
// }

// #[test]
// fn test_after_e2e4() {
//     let mut board = Board::new();
//     board.setup_starting_position();

//     let e2e4 = Move::new(Square(12), Square(28), None, None);
//     board.make_move(&e2e4);

//     assert_eq!(perft(&mut board, 1), 20);
//     assert_eq!(perft(&mut board, 2), 400);
//     assert_eq!(perft(&mut board, 3), 8_902);
//     assert_eq!(perft(&mut board, 4), 197_281);
// }

// #[test]
// fn test_after_nc3() {
//     let mut board = Board::new();
//     board.setup_starting_position();

//     let nc3 = Move::new(Square(1), Square(18), None, None);
//     board.make_move(&nc3);

//     assert_eq!(perft(&mut board, 1), 20);
//     assert_eq!(perft(&mut board, 2), 400);
//     assert_eq!(perft(&mut board, 3), 8_902);
// }

#[test]
fn test_move_unmove_symmetry() {
    let mut board = Board::new();
    board.setup_starting_position();

    let original_perft_3 = perft(&mut board, 3);

    let e2e4 = Move::new(Square(12), Square(28), None, None);
    let undo_e2e4 = board.make_move(&e2e4);
    board.unmake_move(&e2e4, undo_e2e4);

    assert_eq!(
        perft(&mut board, 3),
        original_perft_3,
        "Move-unmove broke position state"
    );

    let nc3 = Move::new(Square(1), Square(18), None, None);
    let undo_nc3 = board.make_move(&nc3);
    board.unmake_move(&nc3, undo_nc3);

    assert_eq!(
        perft(&mut board, 3),
        original_perft_3,
        "Nc3 move-unmove broke position state"
    );
}
