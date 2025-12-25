use crate::board::{Board, BoardRepresentation};
use crate::moves::generate_legal_moves;
use crate::scheduling::{create_e_core_pool, create_p_core_pool, create_pool_for_policy, CorePolicy};
use crate::search::fault_tolerant::should_inject_panic;
use crate::search::minimax::alphabeta;
use crate::search::minimax::MATE_SCORE;
use crate::search::probe::{classify_moves, probe_root_moves};
use crate::types::*;
use rayon::prelude::*;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

static MOVE_COUNTER: AtomicUsize = AtomicUsize::new(0);
static SHOULD_PANIC: AtomicBool = AtomicBool::new(false);

pub struct TwoPhaseConfig {
    pub probe_depth: u8,
    pub p_core_threads: usize,
    pub e_core_threads: usize,
}

impl Default for TwoPhaseConfig {
    fn default() -> Self {
        Self {
            probe_depth: 1,
            p_core_threads: 8,
            e_core_threads: 2,
        }
    }
}

pub fn parallel_search(board: &mut Board, depth: u32) -> (Move, i32) {
    let threads = rayon::current_num_threads();
    parallel_search_with_policy(board, depth, CorePolicy::None, threads, 0.0)
}

pub fn parallel_search_with_policy(board: &mut Board,depth: u32,policy: CorePolicy,threads: usize,mixed_ratio: f32 ) -> (Move, i32) {
    parallel_search_with_fault(board, depth, policy, threads, mixed_ratio, None)
}

pub fn parallel_search_with_fault(board: &mut Board,depth: u32,policy: CorePolicy,threads: usize,mixed_ratio: f32,inject_panic_at: Option<usize>) -> (Move, i32) {
    let current_color = board.to_move();
    let moves = generate_legal_moves(board, current_color);

    if moves.is_empty() {
        let dummy_move = Move::new(Square(0), Square(0), None, None);
        let score = if board.is_in_check(current_color) {
            -MATE_SCORE
        } else {
            0
        };
        return (dummy_move, score);
    }

    // Reset counter
    MOVE_COUNTER.store(0, Ordering::SeqCst);

    // Only enable panic if:
    // 1. inject_panic_at is Some
    // 2. We haven't already panicked (checked via thread_local)
    let should_panic_this_call = inject_panic_at.is_some() && should_inject_panic();
    SHOULD_PANIC.store(should_panic_this_call, Ordering::SeqCst);

    let pool = create_pool_for_policy(policy, threads, mixed_ratio);

    pool.install(|| {
        let results: Vec<(Move, i32)> = moves
            .par_iter()
            .map(|mv| {
                let move_num = MOVE_COUNTER.fetch_add(1, Ordering::SeqCst);

                // Check if we should panic at this move
                if let Some(panic_at) = inject_panic_at {
                    if move_num == panic_at && SHOULD_PANIC.swap(false, Ordering::SeqCst) {
                        // Do real work first (2-ply search)
                        let mut temp_board = board.clone();
                        let temp_undo = temp_board.make_move(mv);
                        let _ = alphabeta(&mut temp_board, 2, i32::MIN + 1, i32::MAX - 1, false);
                        temp_board.unmake_move(mv, temp_undo);

                        panic!("Injected fault at move {} after real work", move_num);
                    }
                }

                // Normal evaluation
                let mut local_board = board.clone();
                let undo = local_board.make_move(mv);
                let score = -alphabeta(
                    &mut local_board,
                    depth.saturating_sub(1),
                    i32::MIN + 1,
                    i32::MAX - 1,
                    false,
                );
                local_board.unmake_move(mv, undo);

                (*mv, score)
            })
            .collect();

        results
            .into_iter()
            .max_by_key(|&(_, score)| score)
            .expect("moves non-empty")
    })
}

pub fn two_phase_search(board: &mut Board, depth: u32, config: &TwoPhaseConfig) -> (Move, i32) {
    let current_color = board.to_move();
    let legal_moves = generate_legal_moves(board, current_color);
    
    if legal_moves.is_empty() {
        let dummy = Move::default();
        let score = if board.is_in_check(current_color) {
            -MATE_SCORE
        } else {
            0
        };
        return (dummy, score);
    }
    
    let probe_depth = config.probe_depth;
    let probed = probe_root_moves(board, &legal_moves, probe_depth);
    
    let (heavy_moves, light_moves) = classify_moves(probed);
    
    println!("  Classification: {} heavy, {} light moves", heavy_moves.len(), light_moves.len());
    
    let p_pool = create_p_core_pool(config.p_core_threads)
        .expect("Failed to create P-core pool");
    
    let (phase1_best_move, phase1_best_score) = if !heavy_moves.is_empty() {
        let result = p_pool.install(|| {
            search_moves_parallel(board, &heavy_moves, depth, i32::MIN + 1)
        });
        println!("  Phase 1 complete: {} (score: {})", result.0.to_algebraic(), result.1);
        result
    } else {
        println!("  Phase 1 skipped: no heavy moves");
        (Move::default(), i32::MIN + 1)
    };
    
    let (phase2_best_move, phase2_best_score) = if !light_moves.is_empty() {
        let e_pool = create_e_core_pool(config.e_core_threads)
            .expect("Failed to create E-core pool");
        
        // Use Phase 1's score as alpha bound (if valid)
        let alpha = if phase1_best_score > i32::MIN + 1 {
            phase1_best_score
        } else {
            i32::MIN + 1
        };
        
        let result = e_pool.install(|| {
            search_moves_parallel(board, &light_moves, depth, alpha)
        });
        println!("  Phase 2 complete: {} (score: {})", result.0.to_algebraic(), result.1);
        result
    } else {
        println!("  Phase 2 skipped: no light moves");
        (Move::default(), i32::MIN + 1)
    };
    
    let (best_move, best_score) = if phase2_best_score > phase1_best_score {
        (phase2_best_move, phase2_best_score)
    } else {
        (phase1_best_move, phase1_best_score)
    };
    
    if best_move.from.0 == 0 && best_move.to.0 == 0 {
        println!("  WARNING: No valid move found, falling back to first legal move");
        let mut fallback_board = board.clone();
        let undo = fallback_board.make_move(&legal_moves[0]);
        let score = -alphabeta(&mut fallback_board, depth.saturating_sub(1), i32::MIN + 1, i32::MAX - 1, false);
        fallback_board.unmake_move(&legal_moves[0], undo);
        return (legal_moves[0], score);
    }
    
    (best_move, best_score)
}

/// Search a set of classified moves in parallel, returning best move and score
fn search_moves_parallel(board: &Board, moves: &[ClassifiedMove], depth: u32, alpha: i32) -> (Move, i32) {
    if moves.is_empty() {
        return (Move::default(), i32::MIN + 1);
    }
    
    let results: Vec<(Move, i32)> = moves
        .par_iter()
        .map(|cm| {
            let mut new_board = board.clone();
            let undo = new_board.make_move(&cm.mv);
            let score = -alphabeta(
                &mut new_board,
                depth.saturating_sub(1),
                -i32::MAX + 1,
                -alpha,
                false,
            );
            new_board.unmake_move(&cm.mv, undo);
            (cm.mv, score)
        })
        .collect();
    
    results
        .into_iter()
        .max_by_key(|(_, score)| *score)
        .unwrap_or((Move::default(), i32::MIN + 1))
}