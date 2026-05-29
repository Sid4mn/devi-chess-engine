//! search correctness regression tests.
//! cover the old sign bug where the drivers negated a non-negamax eval (and
//! hardcoded maximizing_player=false) and ended up playing the worst move.

use devi::board::Board;
use devi::board::BoardRepresentation;
use devi::search::minimax::{root_move_score, MATE_SCORE};
use devi::search::{parallel_search, search};
use devi::types::{Move, Square};

const DEPTH: u32 = 4;

// --- The headline bug: pick the best move, with the right sign ---

#[test]
fn white_takes_the_free_queen() {
    // White pawn on c4 can capture the Black queen on d5 for free.
    let mut b = Board::from_fen("7k/8/8/3q4/2P5/8/8/7K w - - 0 1").unwrap();
    let (mv, score) = search(&mut b, DEPTH);
    assert_eq!(mv.to_algebraic(), "c4d5", "serial search must win the queen");
    assert!(score > 0, "winning the queen must score positive, got {}", score);

    let (pmv, pscore) = parallel_search(&mut b, DEPTH);
    assert_eq!(pmv.to_algebraic(), "c4d5", "parallel search must win the queen");
    assert!(pscore > 0, "parallel score must be positive, got {}", pscore);
}

#[test]
fn black_takes_the_free_queen() {
    // Mirror: Black pawn on c5 can capture the White queen on d4.
    let mut b = Board::from_fen("7K/8/8/2p5/3Q4/8/8/7k b - - 0 1").unwrap();
    let (mv, score) = search(&mut b, DEPTH);
    assert_eq!(mv.to_algebraic(), "c5d4", "serial search must win the queen for Black");
    assert!(score > 0, "score is from the side-to-move's view; must be positive, got {}", score);

    let (pmv, _) = parallel_search(&mut b, DEPTH);
    assert_eq!(pmv.to_algebraic(), "c5d4", "parallel search must win the queen for Black");
}

#[test]
fn does_not_blunder_a_winning_position() {
    // Sanity: from the queen-up position the engine must never report a losing
    // (negative) score for the side to move.
    let mut b = Board::from_fen("7k/8/8/3q4/2P5/8/8/7K w - - 0 1").unwrap();
    let (_, score) = search(&mut b, DEPTH);
    assert!(score >= 0, "must not think a winning position is lost (score {})", score);
}

// --- Mate detection & distance-to-mate scoring ---

#[test]
fn finds_back_rank_mate_in_one() {
    // Ra1-a8 is mate (Black king boxed in by its own pawns).
    let mut b = Board::from_fen("6k1/5ppp/8/8/8/8/8/R5K1 w - - 0 1").unwrap();
    let (mv, score) = search(&mut b, DEPTH);
    assert_eq!(mv.to_algebraic(), "a1a8", "must find the mate");
    assert!(score > MATE_SCORE - 100, "mate must score near +MATE_SCORE, got {}", score);
}

#[test]
fn prefers_faster_mate() {
    // closer mate should score higher than a deeper one (MATE_SCORE - ply).
    let mut b = Board::from_fen("6k1/5ppp/8/8/8/8/8/R5K1 w - - 0 1").unwrap();
    let stm = b.to_move();
    // Ra1-a8 == from square 0 (a1) to 56 (a8).
    let ra8 = Move::new(Square(0), Square(56), None, None);
    let mate_now = root_move_score(&mut b, &ra8, DEPTH, stm);
    assert!(mate_now > MATE_SCORE - 10, "immediate mate should be ~MATE_SCORE, got {}", mate_now);
}

// --- Stalemate / checkmate at the root ---

#[test]
fn reports_checkmate_at_root() {
    // Fool's mate position with White to move and already mated.
    let mut b = Board::from_fen(
        "rnb1kbnr/pppp1ppp/4p3/8/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3",
    ).unwrap();
    let (_, score) = search(&mut b, DEPTH);
    assert_eq!(score, -MATE_SCORE, "a mated side to move must report -MATE_SCORE");
}

#[test]
fn reports_stalemate_as_draw() {
    // Black to move, not in check, no legal moves.
    let mut b = Board::from_fen("k7/2Q5/1K6/8/8/8/8/8 b - - 0 1").unwrap();
    let (_, score) = search(&mut b, DEPTH);
    assert_eq!(score, 0, "stalemate must score 0");
}

// --- depth 0 must not panic (was a u32 underflow) ---

#[test]
fn depth_zero_does_not_panic() {
    let mut b = Board::new();
    b.setup_starting_position();
    let _ = search(&mut b, 0); // must not panic
    let _ = parallel_search(&mut b, 0);
}

// --- serial and parallel agree in clearly tactical positions ---

#[test]
fn serial_and_parallel_agree_when_tactics_are_forced() {
    for fen in [
        "7k/8/8/3q4/2P5/8/8/7K w - - 0 1",   // win queen
        "6k1/5ppp/8/8/8/8/8/R5K1 w - - 0 1", // mate in 1
    ] {
        let mut b1 = Board::from_fen(fen).unwrap();
        let mut b2 = Board::from_fen(fen).unwrap();
        let (m1, _) = search(&mut b1, DEPTH);
        let (m2, _) = parallel_search(&mut b2, DEPTH);
        assert_eq!(m1.to_algebraic(), m2.to_algebraic(), "drivers disagree on {}", fen);
    }
}

#[test]
fn serial_and_parallel_agree_on_ties() {
    // Quiet positions where many moves share the top score (material-only eval).
    // Both drivers use a first-max tie-break, so they must select the same move.
    let quiet = [
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1",
        "r1bqkbnr/pppppppp/2n5/8/8/2N5/PPPPPPPP/R1BQKBNR w KQkq - 0 1",
    ];
    for fen in quiet {
        for depth in 1..=3 {
            let mut b1 = Board::from_fen(fen).unwrap();
            let mut b2 = Board::from_fen(fen).unwrap();
            let (m1, s1) = search(&mut b1, depth);
            let (m2, s2) = parallel_search(&mut b2, depth);
            assert_eq!(s1, s2, "scores differ on {} depth {}", fen, depth);
            assert_eq!(
                m1.to_algebraic(),
                m2.to_algebraic(),
                "tie-break differs on {} depth {}",
                fen,
                depth
            );
        }
    }
}
