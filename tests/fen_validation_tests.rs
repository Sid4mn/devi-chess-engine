//! fen parsing/validation regression tests.
//! from_fen used to accept malformed/impossible positions - silently corrupting
//! the board, or panicking (OOB) on a bogus en passant square during movegen.

use devi::board::{Board, BoardRepresentation};
use devi::moves::generate_legal_moves;

// --- Valid positions still parse ---

#[test]
fn accepts_standard_positions() {
    assert!(Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").is_ok());
    assert!(Board::from_fen(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1"
    )
    .is_ok());
    // legal en passant targets
    assert!(Board::from_fen("rnbqkbnr/ppp1pppp/8/3pP3/8/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 3").is_ok());
    assert!(Board::from_fen("rnbqkbnr/pppp1ppp/8/8/3pP3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 3").is_ok());
}

// --- The en passant panic is closed at the source ---

#[test]
fn rejects_impossible_en_passant_square() {
    // EP target on rank 1 with White to move is impossible and previously made
    // move generation compute Square(to - 8) -> subtract overflow / OOB.
    assert!(Board::from_fen("k1K5/8/8/1P6/8/8/8/8 w - a1 0 1").is_err());
    // EP rank must match side to move (rank 6 for White, rank 3 for Black).
    assert!(Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq e3 0 1").is_err());
    assert!(Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq e6 0 1").is_err());
    // Garbage en passant text must not panic on char subtraction.
    assert!(Board::from_fen("k7/8/8/8/8/8/8/7K w - zz 0 1").is_err());
}

#[test]
fn move_generation_never_panics_on_a_parsed_board() {
    // Whatever from_fen accepts must be safe to generate moves on. The formerly
    // crashing FEN is now rejected, so this just confirms the happy path is fine.
    let mut b = Board::from_fen("8/8/8/1k6/3Pp3/8/8/4K3 b - d3 0 1").unwrap();
    let stm = b.to_move();
    let _ = generate_legal_moves(&mut b, stm); // must not panic
}

// --- Board-shape validation ---

#[test]
fn rejects_under_filled_rank() {
    // "KQ5" describes only 7 squares; previously accepted and silently padded.
    assert!(Board::from_fen("8/8/8/8/8/8/8/KQ5 w - - 0 1").is_err());
}

#[test]
fn rejects_over_filled_rank_and_bad_digits() {
    assert!(Board::from_fen("9/8/8/8/8/8/8/8 w - - 0 1").is_err()); // digit > 8
    assert!(Board::from_fen("8/8/8/8/8/8/8/KQQQQQQQQ w - - 0 1").is_err()); // 9 pieces
    assert!(Board::from_fen("44/8/8/8/8/8/8/8 w - - 0 1").is_ok()); // 4+4 == 8 is fine
    assert!(Board::from_fen("45/8/8/8/8/8/8/8 w - - 0 1").is_err()); // 4+5 == 9
}

#[test]
fn rejects_too_many_kings() {
    // Two white kings -> ambiguous find_king.
    assert!(Board::from_fen("k7/8/8/8/8/8/8/KK6 w - - 0 1").is_err());
    assert!(Board::from_fen("kk6/8/8/8/8/8/8/K7 w - - 0 1").is_err());
}

#[test]
fn rejects_malformed_clocks() {
    assert!(Board::from_fen("k7/8/8/8/8/8/8/7K w - - x 1").is_err());
    assert!(Board::from_fen("k7/8/8/8/8/8/8/7K w - - 0 y").is_err());
}

#[test]
fn rejects_wrong_field_count() {
    assert!(Board::from_fen("8/8/8/8/8/8/8/8").is_err());
    assert!(Board::from_fen("").is_err());
}

// --- to_fen / from_fen round-trips for a valid position ---

#[test]
fn fen_round_trips() {
    let mut b = Board::new();
    b.setup_starting_position();
    let fen = b.to_fen();
    let b2 = Board::from_fen(&fen).unwrap();
    assert_eq!(b2.to_fen(), fen);
}
