// Recovery wrapper correctness tests
use devi::search::fault_tolerant::with_recovery;
use devi::search::{parallel_search, search, minimax::alphabeta};
use devi::board::{Board, BoardRepresentation};
use devi::moves::generate_legal_moves;

#[test]
fn test_recovery_preserves_correctness() {
    let mut board = Board::new();
    board.setup_starting_position();
    
    let (mv1, score1) = parallel_search(&mut board, 5);
    
    let board_clone = board.clone();
    let search_fn = || {
        let mut b = board_clone.clone();
        parallel_search(&mut b, 5)
    };
    let (mv2, score2) = with_recovery(search_fn, Some(5));
    
    assert_eq!(mv1.to_algebraic(), mv2.to_algebraic(), "Move changed");
    assert_eq!(score1, score2, "Score changed");
}

#[test]
fn test_recovery_with_single_thread() {
    let mut board = Board::new();
    board.setup_starting_position();
    
    let board_clone = board.clone();
    let search_fn = || {
        let mut b = board_clone.clone();
        search(&mut b, 4)
    };
    let (mv, score) = with_recovery(search_fn, Some(3));
    
    assert_ne!(score, -1_000_000);
    assert_ne!(mv.to_algebraic(), "a1a1");
}

#[test]
fn test_recovery_consistency_across_panic_positions() {
    let mut board = Board::new();
    board.setup_starting_position();
    
    let (baseline_mv, baseline_score) = parallel_search(&mut board, 4);
    
    // TODO: Sweep full range 0..num_moves
    for panic_at in [0, 5, 10, 15] {
        let board_clone = board.clone();
        let search_fn = || {
            let mut b = board_clone.clone();
            parallel_search(&mut b, 4)
        };
        let (mv, score) = with_recovery(search_fn, Some(panic_at));
        
        assert_eq!(baseline_mv.to_algebraic(), mv.to_algebraic(), 
                   "Move changed at panic_at={}", panic_at);
        assert_eq!(baseline_score, score, 
                   "Score changed at panic_at={}", panic_at);
    }
}

#[test]
fn test_recovery_stress_depths() {
    let mut board = Board::new();
    board.setup_starting_position();
    
    for depth in [4, 5, 6] {
        let (baseline_mv, baseline_score) = parallel_search(&mut board, depth);
        
        let board_clone = board.clone();
        let search_fn = || {
            let mut b = board_clone.clone();
            parallel_search(&mut b, depth)
        };
        let (recovery_mv, recovery_score) = with_recovery(search_fn, Some(5));
        
        assert_eq!(baseline_mv.to_algebraic(), recovery_mv.to_algebraic(),
            "Depth {}: move changed", depth);
        assert_eq!(baseline_score, recovery_score,
            "Depth {}: score changed", depth);
    }
}

#[test]
fn test_recovery_eval_consistency() {
    let mut board = Board::new();
    board.setup_starting_position();
    
    let depth = 5;
    
    let mut b_single = board.clone();
    let (_baseline_mv, baseline_score) = search(&mut b_single, depth);
    
    let board_clone = board.clone();
    let search_fn = || {
        let mut b = board_clone.clone();
        parallel_search(&mut b, depth)
    };
    let (_recovery_mv, recovery_score) = with_recovery(search_fn, Some(5));
    
    assert_eq!(baseline_score, recovery_score,
        "Score mismatch: single={}, multi={}", baseline_score, recovery_score);
}

#[test]
fn test_single_thread_determinism() {
    let mut board = Board::new();
    board.setup_starting_position();
    
    let results: Vec<_> = (0..3)
        .map(|_| {
            let moves = generate_legal_moves(&mut board.clone(), board.to_move());
            let mut best_move = moves[0];
            let mut best_score = -1_000_000;
            
            for mv in moves {
                let mut b = board.clone();
                let _undo = b.make_move(&mv);
                let score = alphabeta(&mut b, 4, -1_000_000, 1_000_000, false);
                if score > best_score {
                    best_score = score;
                    best_move = mv;
                }
            }
            (best_move, best_score)
        })
        .collect();
    
    for i in 1..results.len() {
        assert_eq!(results[0].0.to_algebraic(), results[i].0.to_algebraic(),
            "Non-deterministic: run {} vs run 0", i);
        assert_eq!(results[0].1, results[i].1);
    }
}

#[test]
fn test_recovery_determinism() {
    let mut board = Board::new();
    board.setup_starting_position();
    
    let results: Vec<_> = (0..3)
        .map(|_| {
            let board_clone = board.clone();
            let search_fn = || {
                let mut b = board_clone.clone();
                search(&mut b, 4)
            };
            with_recovery(search_fn, Some(5))
        })
        .collect();
    
    for i in 1..results.len() {
        assert_eq!(results[0].0.to_algebraic(), results[i].0.to_algebraic(),
            "Non-deterministic: run {} vs run 0", i);
        assert_eq!(results[0].1, results[i].1);
    }
}

#[test]
fn test_recovery_without_panic() {
    let mut board = Board::new();
    board.setup_starting_position();
    
    let board_clone = board.clone();
    let search_fn = || {
        let mut b = board_clone.clone();
        search(&mut b, 4)
    };
    
    let (mv, score) = with_recovery(search_fn, None);
    
    assert_ne!(score, -1_000_000);
    assert_ne!(mv.to_algebraic(), "a1a1");
}

#[test]
fn test_recovery_panic_at_zero() {
    let mut board = Board::new();
    board.setup_starting_position();
    
    let board_clone = board.clone();
    let search_fn = || {
        let mut b = board_clone.clone();
        search(&mut b, 3)
    };
    
    let (mv, score) = with_recovery(search_fn, Some(0));
    
    assert_ne!(score, -1_000_000);
    assert_ne!(mv.to_algebraic(), "a1a1");
}