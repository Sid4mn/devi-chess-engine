use crate::board::{Board, BoardRepresentation};
use crate::moves::generate_legal_moves;
use crate::scheduling::{create_e_core_pool, create_p_core_pool, create_pool_for_policy, CorePolicy};
use crate::search::minimax::alphabeta;
use crate::search::minimax::root_move_score;
use crate::search::minimax::MATE_SCORE;
use crate::search::probe::{classify_moves_with_config, probe_root_moves, ClassificationConfig};
use crate::search::recovery::CompletedMoves;
use crate::types::*;
use rayon::prelude::*;
use std::collections::HashSet;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Arc, Mutex};
use std::time::Instant;

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

// fault injection: which stable move indices to crash, plus a per-search set so each fires only once.
#[derive(Clone, Default)]
pub struct InjectionPlan {
    targets: HashSet<usize>,
    fired: Arc<Mutex<HashSet<usize>>>,
}

impl InjectionPlan {
    pub fn none() -> Self {
        Self::default()
    }

    pub fn single(idx: usize) -> Self {
        Self { targets: HashSet::from([idx]), ..Default::default() }
    }

    pub fn set<I: IntoIterator<Item = usize>>(indices: I) -> Self {
        Self { targets: indices.into_iter().collect(), ..Default::default() }
    }
}

// crash a targeted move once, after a little real work. guard drops before the panic so the mutex can't poison.
fn maybe_inject_panic(idx: usize, board: &Board, mv: &Move, plan: &InjectionPlan) {
    if !plan.targets.contains(&idx) {
        return;
    }

    let first_time = {
        let mut fired = plan.fired.lock().unwrap();
        fired.insert(idx) // true only the first time we hit this idx
    };

    if first_time {
        let mut temp_board = board.clone();
        let temp_undo = temp_board.make_move(mv);
        let _ = alphabeta(&mut temp_board, 2, i32::MIN + 1, i32::MAX - 1, false, 0);
        temp_board.unmake_move(mv, temp_undo);

        panic!("Injected fault at move index {}", idx);
    }
}

// what to do with a root move that panics through its whole retry budget.
pub enum GiveUpPolicy {
    ExcludeAndLog, // drop it, log it, return the best of the rest (degraded)
    Abort,         // give up on the whole search
}

// flip this to change behavior. exclude-and-log beats crashing over one bad move.
const GIVE_UP: GiveUpPolicy = GiveUpPolicy::ExcludeAndLog;

// retries of just the failed moves after the initial pass. one covers injected faults; rest is headroom.
const MAX_RETRY_ROUNDS: usize = 2;

#[derive(Clone, Debug)]
pub struct SearchOutcome {
    pub best_move: Move,
    pub score: i32,
    pub degraded: bool, // dropped at least one move
    pub dropped: Vec<Move>,
}

// one parallel pass; catch_unwind per move so a panic doesn't poison the whole batch.
fn run_round(board: &Board, moves: &[Move], pending: &[usize], depth: u32, color: Color, inject: &InjectionPlan, pool: &rayon::ThreadPool) -> Vec<(usize, Result<(Move, i32), ()>)> {
    pool.install(|| {
        pending
            .par_iter()
            .map(|&idx| {
                let mv = &moves[idx];
                let outcome = catch_unwind(AssertUnwindSafe(|| {
                    maybe_inject_panic(idx, board, mv, inject);
                    // root-side-relative score so we can just take the max later
                    let mut local_board = board.clone();
                    root_move_score(&mut local_board, mv, depth, color)
                }));

                match outcome {
                    Ok(score) => (idx, Ok((*mv, score))),
                    Err(_) => (idx, Err(())),
                }
            })
            .collect()
    })
}

// reduce the completed moves to a best move, dealing with any that never finished.
fn finalize(completed: CompletedMoves, moves: &[Move], failed: &[usize]) -> SearchOutcome {
    if !failed.is_empty() {
        match GIVE_UP {
            GiveUpPolicy::ExcludeAndLog => {
                for &idx in failed {
                    eprintln!(
                        "WARNING: dropping root move {} - failed every retry (result is DEGRADED)",
                        moves[idx].to_algebraic()
                    );
                }
            }
            GiveUpPolicy::Abort => {
                panic!("search aborted: {} root move(s) failed permanently", failed.len());
            }
        }
    }

    match completed.best() {
        Some((best_move, score)) => SearchOutcome {
            best_move,
            score,
            degraded: !failed.is_empty(),
            dropped: failed.iter().map(|&idx| moves[idx]).collect(),
        },
        // every move failed - nothing to return.
        None => panic!("search wipeout: all {} root moves failed", moves.len()),
    }
}

// back-compat entry: keeps the old Option<usize> shape so existing callers don't change.
pub fn parallel_search_with_fault(board: &mut Board,depth: u32,policy: CorePolicy,threads: usize,mixed_ratio: f32,inject_panic_at: Option<usize>) -> (Move, i32) {
    let inject = match inject_panic_at {
        Some(idx) => InjectionPlan::single(idx),
        None => InjectionPlan::none(),
    };
    let outcome = parallel_search_with_fault_set(board, depth, policy, threads, mixed_ratio, inject);
    (outcome.best_move, outcome.score)
}

// real driver: search each root move in parallel, isolate + retry the ones that panic.
pub fn parallel_search_with_fault_set(board: &mut Board,depth: u32,policy: CorePolicy,threads: usize,mixed_ratio: f32,inject: InjectionPlan) -> SearchOutcome {
    let current_color = board.to_move();
    let moves = generate_legal_moves(board, current_color);

    if moves.is_empty() {
        let dummy_move = Move::new(Square(0), Square(0), None, None);
        let score = if board.is_in_check(current_color) {
            -MATE_SCORE
        } else {
            0
        };
        return SearchOutcome { best_move: dummy_move, score, degraded: false, dropped: Vec::new() };
    }

    let pool = create_pool_for_policy(policy, threads, mixed_ratio);

    let mut completed = CompletedMoves::new(moves.len());
    let mut pending: Vec<usize> = (0..moves.len()).collect();
    let mut round = 0;

    // retry only the moves that failed; completed ones stay recorded so we never recompute them.
    let failed = loop {
        let batch = run_round(board, &moves, &pending, depth, current_color, &inject, &pool);

        let mut failed = Vec::new();
        for (idx, res) in batch {
            match res {
                Ok((mv, score)) => completed.record(idx, mv, score),
                Err(()) => failed.push(idx),
            }
        }

        if failed.is_empty() || round >= MAX_RETRY_ROUNDS {
            break failed;
        }
        pending = failed;
        round += 1;
    };

    finalize(completed, &moves, &failed)
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
        let root_color = board.to_move();
        let mut fallback_board = board.clone();
        let score = root_move_score(&mut fallback_board, &legal_moves[0], depth, root_color);
        (legal_moves[0], score)
    } else {
        (best_move, best_score)
    };
    
    metrics.total_time_ms = total_start.elapsed().as_secs_f64() * 1000.0;
    metrics.best_move = best_move.to_algebraic();
    metrics.score = best_score;
    
    (best_move, best_score, metrics)
}

// search a batch of classified moves in parallel, return best (move, score).
// _alpha used to be the earlier phase's best for cross-phase pruning, but mixing
// that with the full-window per-move search was wrong, so we ignore it now and
// just search each move full-window. keeps the signature for the callers.
fn search_moves_parallel(board: &Board, moves: &[ClassifiedMove], depth: u32, _alpha: i32) -> (Move, i32) {
    if moves.is_empty() {
        return (Move::default(), i32::MIN + 1);
    }

    let root_color = board.to_move();

    let results: Vec<(Move, i32)> = moves
        .par_iter()
        .map(|cm| {
            let mut new_board = board.clone();
            let score = root_move_score(&mut new_board, &cm.mv, depth, root_color);
            (cm.mv, score)
        })
        .collect();

    // first-max, same as the other drivers
    results
        .into_iter()
        .reduce(|best, cur| if cur.1 > best.1 { cur } else { best })
        .unwrap_or((Move::default(), i32::MIN + 1))
}