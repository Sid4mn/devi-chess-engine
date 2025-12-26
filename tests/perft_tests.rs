//! Perft tests for move generation correctness.
//! Reference: https://www.chessprogramming.org/Perft_Results

use devi::board::{Board, BoardRepresentation};
use devi::moves::{perft, perft_detailed, verify_positions};
use devi::types::*;

// --- Standard positions (CPW) ---

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
#[cfg(not(debug_assertions))]
fn test_perft_starting_position_depth6() {
    let mut board = Board::new();
    board.setup_starting_position();
    assert_eq!(perft(&mut board, 6), 119_060_324);
}

#[test]
fn test_perft_kiwipete() {
    let mut board = Board::from_fen(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
    ).unwrap();
    assert_eq!(perft(&mut board, 1), 48);
    assert_eq!(perft(&mut board, 2), 2039);
    assert_eq!(perft(&mut board, 3), 97862);
    assert_eq!(perft(&mut board, 4), 4085603);
}

#[test]
fn test_perft_position3() {
    let mut board = Board::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").unwrap();
    assert_eq!(perft(&mut board, 1), 14);
    assert_eq!(perft(&mut board, 2), 191);
    assert_eq!(perft(&mut board, 3), 2812);
    assert_eq!(perft(&mut board, 4), 43238);
    assert_eq!(perft(&mut board, 5), 674624);
}

#[test]
fn test_perft_position4() {
    let mut board = Board::from_fen(
        "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
    ).unwrap();
    assert_eq!(perft(&mut board, 1), 6);
    assert_eq!(perft(&mut board, 2), 264);
    assert_eq!(perft(&mut board, 3), 9467);
    assert_eq!(perft(&mut board, 4), 422333);
}

#[test]
fn test_perft_position5() {
    let mut board = Board::from_fen(
        "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
    ).unwrap();
    assert_eq!(perft(&mut board, 1), 44);
    assert_eq!(perft(&mut board, 2), 1486);
    assert_eq!(perft(&mut board, 3), 62379);
    assert_eq!(perft(&mut board, 4), 2103487);
}

#[test]
fn test_perft_position6() {
    let mut board = Board::from_fen(
        "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
    ).unwrap();
    assert_eq!(perft(&mut board, 1), 46);
    assert_eq!(perft(&mut board, 2), 2079);
    assert_eq!(perft(&mut board, 3), 89890);
    assert_eq!(perft(&mut board, 4), 3894594);
}

// --- Deep tests (release only) ---

#[test]
#[cfg(not(debug_assertions))]
fn test_perft_kiwipete_depth5() {
    let mut board = Board::from_fen(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
    ).unwrap();
    assert_eq!(perft(&mut board, 5), 193690690);
}

#[test]
#[cfg(not(debug_assertions))]
fn test_perft_position3_depth6() {
    let mut board = Board::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").unwrap();
    assert_eq!(perft(&mut board, 6), 11030083);
}

#[test]
#[cfg(not(debug_assertions))]
fn test_perft_position4_depth5() {
    let mut board = Board::from_fen(
        "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
    ).unwrap();
    assert_eq!(perft(&mut board, 5), 15833292);
}

#[test]
#[cfg(not(debug_assertions))]
fn test_perft_position5_depth5() {
    let mut board = Board::from_fen(
        "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
    ).unwrap();
    assert_eq!(perft(&mut board, 5), 89941194);
}

#[test]
#[cfg(not(debug_assertions))]
fn test_perft_position6_depth5() {
    let mut board = Board::from_fen(
        "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
    ).unwrap();
    assert_eq!(perft(&mut board, 5), 164075551);
}

// --- Detailed statistics ---

#[test]
fn test_perft_starting_detailed_depth3() {
    let mut board = Board::new();
    board.setup_starting_position();
    let stats = perft_detailed(&mut board, 3);
    assert_eq!(stats.nodes, 8902);
    assert_eq!(stats.captures, 34);
    assert_eq!(stats.en_passant, 0);
    assert_eq!(stats.castles, 0);
    assert_eq!(stats.promotions, 0);
    assert_eq!(stats.checks, 12);
    assert_eq!(stats.checkmates, 0);
}

#[test]
fn test_perft_starting_detailed_depth4() {
    let mut board = Board::new();
    board.setup_starting_position();
    let stats = perft_detailed(&mut board, 4);
    assert_eq!(stats.nodes, 197281);
    assert_eq!(stats.captures, 1576);
    assert_eq!(stats.en_passant, 0);
    assert_eq!(stats.castles, 0);
    assert_eq!(stats.promotions, 0);
    assert_eq!(stats.checks, 469);
    assert_eq!(stats.checkmates, 8);
}

#[test]
#[cfg(not(debug_assertions))]
fn test_perft_starting_detailed_depth5() {
    let mut board = Board::new();
    board.setup_starting_position();
    let stats = perft_detailed(&mut board, 5);
    assert_eq!(stats.nodes, 4865609);
    assert_eq!(stats.captures, 82719);
    assert_eq!(stats.en_passant, 258);
    assert_eq!(stats.castles, 0);
    assert_eq!(stats.promotions, 0);
    assert_eq!(stats.checks, 27351);
    assert_eq!(stats.checkmates, 347);
}

#[test]
fn test_verify_positions_api() {
    let positions = [
        ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 1, 20u64),
        ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 2, 400u64),
        ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 3, 8902u64),
        ("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", 1, 48u64),
        ("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", 2, 2039u64),
    ];
    assert!(verify_positions(&positions));
}

#[test]
fn test_perft_kiwipete_detailed() {
    let mut board = Board::from_fen(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
    ).unwrap();
    let stats1 = perft_detailed(&mut board, 1);
    assert_eq!(stats1.nodes, 48);
    assert_eq!(stats1.captures, 8);
    assert_eq!(stats1.castles, 2);
    let stats2 = perft_detailed(&mut board, 2);
    assert_eq!(stats2.nodes, 2039);
}

// --- Move/unmove symmetry ---

#[test]
fn test_move_unmove_symmetry() {
    let mut board = Board::new();
    board.setup_starting_position();
    let original = perft(&mut board, 3);

    let e2e4 = Move::new(Square(12), Square(28), None, None);
    let undo = board.make_move(&e2e4);
    board.unmake_move(&e2e4, undo);
    assert_eq!(perft(&mut board, 3), original);

    let nc3 = Move::new(Square(1), Square(18), None, None);
    let undo = board.make_move(&nc3);
    board.unmake_move(&nc3, undo);
    assert_eq!(perft(&mut board, 3), original);
}

// --- Tricky positions ---

#[test]
fn test_perft_talkchess() {
    let mut board = Board::from_fen(
        "rnbqkb1r/pp1p1ppp/2p5/4P3/2B5/8/PPP1NnPP/RNBQK2R w KQkq - 0 6",
    ).unwrap();
    assert_eq!(perft(&mut board, 1), 42);
    assert_eq!(perft(&mut board, 2), 1352);
    assert_eq!(perft(&mut board, 3), 53392);
}

#[test]
fn test_perft_position4_mirrored() {
    let mut board = Board::from_fen(
        "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1",
    ).unwrap();
    assert_eq!(perft(&mut board, 1), 6);
    assert_eq!(perft(&mut board, 2), 264);
    assert_eq!(perft(&mut board, 3), 9467);
    assert_eq!(perft(&mut board, 4), 422333);
}

#[test]
fn test_perft_ep_discovered_check() {
    let mut board = Board::from_fen("8/2p5/3p4/KP5r/1R2Pp1k/8/6P1/8 b - e3 0 1").unwrap();
    assert_eq!(perft(&mut board, 1), 16);
    assert_eq!(perft(&mut board, 2), 177);
    assert_eq!(perft(&mut board, 3), 2748);
}

#[test]
fn test_perft_promotion_check() {
    let mut board = Board::from_fen("n1n5/1Pk5/8/8/8/8/5Kp1/5N1N w - - 0 1").unwrap();
    assert_eq!(perft(&mut board, 1), 24);
    assert_eq!(perft(&mut board, 2), 421);
    assert_eq!(perft(&mut board, 3), 7421);
    assert_eq!(perft(&mut board, 4), 124608);
}

#[test]
fn test_perft_castling_rook_capture() {
    let mut board = Board::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1").unwrap();
    assert_eq!(perft(&mut board, 1), 26);
    assert_eq!(perft(&mut board, 2), 568);
    assert_eq!(perft(&mut board, 3), 13744);
    assert_eq!(perft(&mut board, 4), 314346);
}

#[test]
fn test_perft_illegal_ep() {
    let mut board = Board::from_fen("8/8/8/8/k2Pp2Q/8/8/3K4 b - d3 0 1").unwrap();
    assert_eq!(perft(&mut board, 1), 6);
}

#[test]
fn test_perft_max_moves() {
    let mut board = Board::from_fen(
        "R6R/3Q4/1Q4Q1/4Q3/2Q4Q/Q4Q2/pp1Q4/kBNN1KB1 w - - 0 1",
    ).unwrap();
    assert!(perft(&mut board, 1) >= 200);
}

#[test]
fn test_perft_complex_middlegame() {
    let mut board = Board::from_fen(
        "r1bq1rk1/pp2ppbp/2np1np1/8/3NP3/2N1BP2/PPPQ2PP/R3KB1R w KQ - 0 9",
    ).unwrap();
    assert_eq!(perft(&mut board, 1), 45);
    assert_eq!(perft(&mut board, 2), 1602);
}

#[test]
fn test_perft_sicilian_dragon() {
    let mut board = Board::from_fen(
        "r1bqk2r/pp2ppbp/2np1np1/8/3NP3/2N1B3/PPPQ1PPP/R3KB1R w KQkq - 0 8",
    ).unwrap();
    assert_eq!(perft(&mut board, 1), 43);
    assert_eq!(perft(&mut board, 2), 1609);
}

// --- Castling edge cases ---

#[test]
fn test_castling_both_sides_available() {
    let mut board = Board::from_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1").unwrap();
    assert!(perft(&mut board, 1) >= 18);
}

#[test]
fn test_castling_blocked_by_piece() {
    let mut board = Board::from_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K1NR w KQkq - 0 1").unwrap();
    assert!(perft(&mut board, 1) > 0);
}

#[test]
fn test_castling_blocked_by_pawn_attack_f1() {
    let mut board = Board::from_fen(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q2/PPPBBPp1/1R2K2R w Kkq - 0 2",
    ).unwrap();
    assert_eq!(perft(&mut board, 1), 49);
}

#[test]
fn test_castling_blocked_by_pawn_attack_d1() {
    let mut board = Board::from_fen("r3k2r/pppppppp/8/8/8/8/PPpPPPPP/R3K2R w KQkq - 0 1").unwrap();
    assert!(perft(&mut board, 1) > 0);
}

#[test]
fn test_castling_through_check() {
    let mut board = Board::from_fen("5r2/8/8/8/8/8/8/R3K2R w KQ - 0 1").unwrap();
    assert!(perft(&mut board, 1) > 0);
}

#[test]
fn test_castling_out_of_check() {
    let mut board = Board::from_fen("4r3/8/8/8/8/8/8/R3K2R w KQ - 0 1").unwrap();
    assert!(perft(&mut board, 1) > 0);
}

#[test]
fn test_castling_into_check() {
    let mut board = Board::from_fen("6r1/8/8/8/8/8/8/R3K2R w KQ - 0 1").unwrap();
    assert!(perft(&mut board, 1) > 0);
}

#[test]
fn test_black_castling_blocked_by_pawn() {
    let mut board = Board::from_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R b KQkq - 0 1").unwrap();
    assert!(perft(&mut board, 1) >= 16);
}

#[test]
fn test_castling_lost_after_rook_move() {
    let mut board = Board::from_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1").unwrap();
    assert!(perft(&mut board, 1) > 0);
}

#[test]
fn test_castling_kingside_only() {
    let mut board = Board::from_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/4K2R w Kk - 0 1").unwrap();
    assert!(perft(&mut board, 1) > 0);
}

#[test]
fn test_castling_queenside_only() {
    let mut board = Board::from_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K3 w Qq - 0 1").unwrap();
    assert!(perft(&mut board, 1) > 0);
}

// --- En passant edge cases ---

#[test]
fn test_en_passant_horizontal_pin() {
    let mut board = Board::from_fen("8/8/8/8/k2Pp2Q/8/8/3K4 b - d3 0 1").unwrap();
    assert_eq!(perft(&mut board, 1), 6);
}

#[test]
fn test_en_passant_rook_pin() {
    let mut board = Board::from_fen("8/8/8/8/k2Pp2R/8/8/3K4 b - d3 0 1").unwrap();
    assert_eq!(perft(&mut board, 1), 6);
}

#[test]
fn test_en_passant_legal() {
    let mut board = Board::from_fen("8/8/8/1k6/3Pp3/8/8/4K3 b - d3 0 1").unwrap();
    assert!(perft(&mut board, 1) > 6);
}

#[test]
fn test_en_passant_multiple_pawns() {
    let mut board = Board::from_fen("8/8/8/pPpPpPpP/P1P1P1P1/8/8/k1K5 w - a6 0 1").unwrap();
    assert!(perft(&mut board, 1) > 0);
}

#[test]
fn test_en_passant_white_captures() {
    let mut board = Board::from_fen(
        "rnbqkbnr/ppp1pppp/8/3pP3/8/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 3",
    ).unwrap();
    assert!(perft(&mut board, 1) >= 30);
}

#[test]
fn test_en_passant_black_captures() {
    let mut board = Board::from_fen(
        "rnbqkbnr/ppp1pppp/8/8/3pP3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 3",
    ).unwrap();
    assert!(perft(&mut board, 1) >= 30);
}

// --- Promotion edge cases ---

#[test]
fn test_promotion_all_types() {
    let mut board = Board::from_fen("8/P7/8/8/8/8/8/k1K5 w - - 0 1").unwrap();
    assert!(perft(&mut board, 1) >= 4);
}

#[test]
fn test_promotion_capture() {
    let mut board = Board::from_fen("r1n5/1P6/8/8/8/8/8/k1K5 w - - 0 1").unwrap();
    assert!(perft(&mut board, 1) >= 8);
}

#[test]
fn test_promotion_multiple_pawns() {
    let mut board = Board::from_fen("n1n5/PPPk4/8/8/8/8/4Kppp/5N1N b - - 0 1").unwrap();
    assert_eq!(perft(&mut board, 1), 24);
    assert_eq!(perft(&mut board, 2), 496);
    assert_eq!(perft(&mut board, 3), 9483);
}

#[test]
fn test_promotion_black() {
    let mut board = Board::from_fen("k1K5/8/8/8/8/8/p7/8 b - - 0 1").unwrap();
    assert!(perft(&mut board, 1) >= 4);
}

#[test]
fn test_promotion_with_check() {
    let mut board = Board::from_fen("8/P7/8/8/8/8/8/k6K w - - 0 1").unwrap();
    assert!(perft(&mut board, 1) > 0);
}

// --- Pins and checks ---

#[test]
fn test_pinned_piece_cannot_move() {
    let mut board = Board::from_fen("8/8/8/8/1k6/8/2P5/K3r3 w - - 0 1").unwrap();
    assert_eq!(perft(&mut board, 1), 2);
}

#[test]
fn test_pinned_piece_captures_pinner() {
    let mut board = Board::from_fen("8/8/8/8/8/8/1B6/K2r3k w - - 0 1").unwrap();
    assert!(perft(&mut board, 1) > 0);
}

#[test]
fn test_double_check_king_only() {
    let mut board = Board::from_fen("8/8/8/8/8/2b5/1q6/K7 w - - 0 1").unwrap();
    assert!(perft(&mut board, 1) <= 3);
}

#[test]
fn test_discovered_check() {
    let mut board = Board::from_fen("8/8/8/8/k7/8/R1N5/K7 w - - 0 1").unwrap();
    assert!(perft(&mut board, 1) > 0);
}

// --- Terminal positions ---

#[test]
fn test_stalemate() {
    let mut board = Board::from_fen("k7/2Q5/1K6/8/8/8/8/8 b - - 0 1").unwrap();
    assert_eq!(perft(&mut board, 1), 0);
}

#[test]
fn test_checkmate_fools_mate() {
    let mut board = Board::from_fen(
        "rnb1kbnr/pppp1ppp/4p3/8/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3",
    ).unwrap();
    assert_eq!(perft(&mut board, 1), 0);
}

#[test]
fn test_checkmate_back_rank() {
    let mut board = Board::from_fen("3R2k1/5ppp/8/8/8/8/8/4K3 b - - 0 1").unwrap();
    assert_eq!(perft(&mut board, 1), 0);
}

#[test]
fn test_near_stalemate() {
    let mut board = Board::from_fen("1k6/8/1Q6/8/8/8/8/1K6 b - - 0 1").unwrap();
    assert!(perft(&mut board, 1) > 0);
}

// --- FEN parsing ---

#[test]
fn test_fen_roundtrip() {
    let mut board = Board::new();
    board.setup_starting_position();
    let fen = board.to_fen();
    let mut board2 = Board::from_fen(&fen).unwrap();
    assert_eq!(perft(&mut board, 3), perft(&mut board2, 3));
}

#[test]
fn test_fen_all_castling() {
    let mut board = Board::from_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1").unwrap();
    assert!(perft(&mut board, 1) > 0);
}

#[test]
fn test_fen_no_castling() {
    let mut board = Board::from_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w - - 0 1").unwrap();
    assert!(perft(&mut board, 1) > 0);
}

#[test]
fn test_fen_en_passant() {
    let mut board = Board::from_fen(
        "rnbqkbnr/pppp1ppp/8/4pP2/8/8/PPPPP1PP/RNBQKBNR w KQkq e6 0 3",
    ).unwrap();
    assert!(perft(&mut board, 1) > 0);
}

// --- Opening positions ---

#[test]
fn test_italian_game() {
    let mut board = Board::from_fen(
        "r1bqk2r/pppp1ppp/2n2n2/2b1p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4",
    ).unwrap();
    assert!(perft(&mut board, 1) > 0);
    assert!(perft(&mut board, 2) > 0);
}

#[test]
fn test_sicilian_defense() {
    let mut board = Board::from_fen(
        "rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 2",
    ).unwrap();
    assert!(perft(&mut board, 1) >= 29);
}

#[test]
fn test_middlegame_position() {
    let mut board = Board::from_fen(
        "r2qkb1r/ppp2ppp/2np1n2/4p1B1/2B1P1b1/3P1N2/PPP2PPP/RN1QK2R w KQkq - 0 6",
    ).unwrap();
    assert!(perft(&mut board, 1) > 30);
    assert!(perft(&mut board, 2) > 1000);
}

// --- Endgames ---

#[test]
fn test_endgame_kp_vs_k() {
    let mut board = Board::from_fen("8/5k2/8/5P2/8/8/8/4K3 w - - 0 1").unwrap();
    assert!(perft(&mut board, 1) > 0);
    assert!(perft(&mut board, 4) > 0);
}

#[test]
fn test_endgame_kr_vs_k() {
    let mut board = Board::from_fen("8/8/8/8/8/5k2/8/4K2R w - - 0 1").unwrap();
    assert!(perft(&mut board, 1) > 10);
}

// --- Black to move ---

#[test]
fn test_black_castling() {
    let mut board = Board::from_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R b KQkq - 0 1").unwrap();
    assert!(perft(&mut board, 1) >= 16);
}

#[test]
fn test_black_in_check() {
    let mut board = Board::from_fen(
        "rnbqkbnr/ppppp1pp/5p2/7Q/4P3/8/PPPP1PPP/RNB1KBNR b KQkq - 1 2",
    ).unwrap();
    assert!(perft(&mut board, 1) > 0);
}

#[test]
fn test_white_pawn_blocks_black_castle() {
    let mut board = Board::from_fen(
        "r3k2r/pppppppp/5P2/8/8/8/PPPPP1PP/R3K2R b KQkq - 0 1",
    ).unwrap();
    assert!(perft(&mut board, 1) > 0);
}
