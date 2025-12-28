use crate::board::{Board, BoardRepresentation};
use crate::moves::generate_legal_moves;
use crate::scheduling::{create_e_core_pool, create_p_core_pool, create_pool_for_policy, CorePolicy};
use crate::search::fault_tolerant::should_inject_panic;
use crate::search::minimax::alphabeta;
use crate::search::minimax::MATE_SCORE;
use crate::search::probe::{classify_moves_with_config, probe_root_moves, ClassificationConfig};
use crate::types::*;
use rayon::prelude::*;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Instant;

static MOVE_COUNTER: AtomicUsize = AtomicUsize::new(0);
static SHOULD_PANIC: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Copy, Debug)]
pub struct TwoPhaseConfig {
    pub probe_depth: u8,
    pub p_core_threads: usize,
    pub e_core_threads: usize,
    pub classification: ClassificationConfig,
}

impl Default for TwoPhaseConfig {
    fn default() -> Self {
        Self {
            probe_depth: 1,
            p_core_threads: 8,
            e_core_threads: 2,
            classification: ClassificationConfig::default(),
        }
    }
}

/// Decide whether to use two-phase scheduling based on move count.
/// Returns optimal config if beneficial, None if baseline is better.
pub fn should_use_two_phase(legal_move_count: usize) -> Option<TwoPhaseConfig> {
    match legal_move_count {
        0..=10 => None, // Skip two-phase: too few moves to classify
        11..=25 => Some(TwoPhaseConfig {
            probe_depth: 2,
            classification: ClassificationConfig { heavy_ratio: 0.6, light_threshold: 0.3 },
            ..Default::default()
        }),
        _ => Some(TwoPhaseConfig {
            probe_depth: 1,
            classification: ClassificationConfig { heavy_ratio: 0.8, light_threshold: 0.3 },
            ..Default::default()
        }),
    }
}

/// Detailed timing metrics from two-phase search
#[derive(Clone, Debug, Default)]
pub struct TwoPhaseMetrics {
    pub probe_time_ms: f64,
    pub phase1_time_ms: f64,
    pub phase2_time_ms: f64,
    pub total_time_ms: f64,
    pub heavy_move_count: usize,
    pub light_move_count: usize,
    pub best_move: String,
    pub score: i32,
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
    let (mv, score, _) = two_phase_search_with_metrics(board, depth, config);
    (mv, score)
}

/// Two-phase search with detailed timing metrics for benchmarking
pub fn two_phase_search_with_metrics(
    board: &mut Board,
    depth: u32,
    config: &TwoPhaseConfig,
) -> (Move, i32, TwoPhaseMetrics) {
    let total_start = Instant::now();
    let mut metrics = TwoPhaseMetrics::default();
    
    let current_color = board.to_move();
    let legal_moves = generate_legal_moves(board, current_color);
    
    if legal_moves.is_empty() {
        let dummy = Move::default();
        let score = if board.is_in_check(current_color) {
            -MATE_SCORE
        } else {
            0
        };
        metrics.total_time_ms = total_start.elapsed().as_secs_f64() * 1000.0;
        return (dummy, score, metrics);
    }
    
    // Probe phase
    let probe_start = Instant::now();
    let probed = probe_root_moves(board, &legal_moves, config.probe_depth);
    metrics.probe_time_ms = probe_start.elapsed().as_secs_f64() * 1000.0;
    
    let (heavy_moves, light_moves) = classify_moves_with_config(probed, &config.classification);
    metrics.heavy_move_count = heavy_moves.len();
    metrics.light_move_count = light_moves.len();
    
    let p_pool = create_p_core_pool(config.p_core_threads)
        .expect("Failed to create P-core pool");
    
    // Phase 1: Heavy moves on P-cores
    let phase1_start = Instant::now();
    let (phase1_best_move, phase1_best_score) = if !heavy_moves.is_empty() {
        p_pool.install(|| {
            search_moves_parallel(board, &heavy_moves, depth, i32::MIN + 1)
        })
    } else {
        (Move::default(), i32::MIN + 1)
    };
    metrics.phase1_time_ms = phase1_start.elapsed().as_secs_f64() * 1000.0;
    
    // Phase 2: Light moves on E-cores
    let phase2_start = Instant::now();
    let (phase2_best_move, phase2_best_score) = if !light_moves.is_empty() {
        let e_pool = create_e_core_pool(config.e_core_threads)
            .expect("Failed to create E-core pool");
        
        let alpha = if phase1_best_score > i32::MIN + 1 {
            phase1_best_score
        } else {
            i32::MIN + 1
        };
        
        e_pool.install(|| {
            search_moves_parallel(board, &light_moves, depth, alpha)
        })
    } else {
        (Move::default(), i32::MIN + 1)
    };
    metrics.phase2_time_ms = phase2_start.elapsed().as_secs_f64() * 1000.0;
    
    let (best_move, best_score) = if phase2_best_score > phase1_best_score {
        (phase2_best_move, phase2_best_score)
    } else {
        (phase1_best_move, phase1_best_score)
    };
    
    // Fallback if no valid move found
    let (best_move, best_score) = if best_move.from.0 == 0 && best_move.to.0 == 0 {
        let mut fallback_board = board.clone();
        let undo = fallback_board.make_move(&legal_moves[0]);
        let score = -alphabeta(&mut fallback_board, depth.saturating_sub(1), i32::MIN + 1, i32::MAX - 1, false);
        fallback_board.unmake_move(&legal_moves[0], undo);
        (legal_moves[0], score)
    } else {
        (best_move, best_score)
    };
    
    metrics.total_time_ms = total_start.elapsed().as_secs_f64() * 1000.0;
    metrics.best_move = best_move.to_algebraic();
    metrics.score = best_score;
    
    (best_move, best_score, metrics)
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