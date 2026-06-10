use crate::board::{Board, BoardRepresentation};
use crate::moves::generate_legal_moves;
use crate::scheduling::{create_e_core_pool, create_p_core_pool, create_pool_for_policy, CorePolicy};
use crate::search::top_two_root_moves_shallow;
use crate::search::minimax::alphabeta;
use crate::search::minimax::root_move_score;
use crate::search::minimax::root_move_score_bounded;
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
            // depth-1 probing is noise: every move has ~the same reply count, so it can't
            // tell heavy from light. depth 2 is where the ranking signal actually shows up.
            probe_depth: 2,
            p_core_threads: 8,
            e_core_threads: 2,
            classification: ClassificationConfig::default(),
        }
    }
}

/* should_use_two_phase used to live here, picking a config by move count. deleted after
   the reality check - two-phase doesn't beat plain work-stealing on this chip at any
   move count, so there's no "beneficial" config to return.
   see releases/v0.6.0/reality_check.md */

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

/* one parallel pass; catch_unwind per move so a panic doesn't poison the whole batch.
   `best_root` is the shared alpha (root POV) for this batch. it's static within a batch
   on purpose: a live shared bound would make tie-breaks depend on thread timing, and the
   engine promises serial == parallel. the driver tightens the bound BETWEEN chunks
   instead. i32::MIN + 1 means "no bound yet" and degenerates to a full-window search. */
fn run_round(board: &Board, moves: &[Move], pending: &[usize], depth: u32, color: Color, best_root: i32, inject: &InjectionPlan, pool: &rayon::ThreadPool) -> Vec<(usize, Result<(Move, i32), ()>)> {
    pool.install(|| {
        pending
            .par_iter()
            .map(|&idx| {
                let mv = &moves[idx];
                let outcome = catch_unwind(AssertUnwindSafe(|| {
                    maybe_inject_panic(idx, board, mv, inject);
                    // root-side-relative score so we can just take the max later
                    let mut local_board = board.clone();
                    root_move_score_bounded(&mut local_board, mv, depth, color, best_root)
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

    /* Young Brothers Wait: search one likely-best move FIRST (full window) to seed a tight
       alpha, then search the rest in parallel bounded by it so they can prune. The seed is
       picked by a cheap shallow search (best-at-shallow is almost always best-at-full), and
       ordering only affects speed, never the result. best_root stays i32::MIN + 1 if seeding fails. */
    /* the seed pick + seed search run inside the policy pool too, not on the caller thread.
       they used to run outside it, which meant a big serial chunk always ran at foreground
       qos no matter what --core-policy said - the policy benchmarks were only measuring the
       post-seed batch. */
    let shallow_depth = depth.saturating_sub(2).clamp(1, 4);
    let (seed_idx, runner_up) = pool.install(|| top_two_root_moves_shallow(board, &moves, current_color, shallow_depth));
    let mut best_root = i32::MIN + 1;

    /* Seed is fault-isolated, and a seed panic no longer means "skip seeding entirely" -
       that silently cost ~3x, because the whole batch then ran full-window with no shared
       bound. the fallback is the shallow runner-up specifically: a near-best bound still
       prunes well, an arbitrary replacement move wouldn't. both dead -> full-window. */
    let mut candidates = vec![seed_idx];
    if let Some(r) = runner_up {
        candidates.push(r);
    }
    let mut seeded: Option<usize> = None;
    for idx in candidates {
        let mv = &moves[idx];
        let res = pool.install(|| catch_unwind(AssertUnwindSafe(|| {
            maybe_inject_panic(idx, board, mv, &inject);
            let mut local = board.clone();
            root_move_score(&mut local, mv, depth, current_color)
        })));
        if let Ok(score) = res {
            completed.record(idx, *mv, score);
            best_root = score;
            seeded = Some(idx);
            break;
        }
        // panicked - fall through to the fallback candidate (a crashed seed lands back in
        // pending and gets retried in the batch like any other move)
    }

    if std::env::var("DEVI_DEBUG_SEED").is_ok() {
        eprintln!("DEBUG seed_idx={} ({}) runner_up={:?} seeded={:?} best_root={}",
            seed_idx, moves[seed_idx].to_algebraic(), runner_up, seeded, best_root);
    }

    // Everything still to do: all moves except a successfully-seeded seed.
    let mut pending: Vec<usize> = (0..moves.len())
        .filter(|&i| seeded != Some(i))
        .collect();
    let mut round = 0;

    /* retry only the moves that failed; completed ones stay recorded so we never recompute
       them. each round runs in index-ordered chunks of one pool's worth, and the bound is
       re-raised from completed scores between chunks. that keeps everything deterministic
       (scores never depend on thread timing, ties resolve in move order like the serial
       search) while still letting a weak bound recover mid-round - which is what caps the
       cost of losing the seed move to a fault. */
    let chunk_size = threads.max(1);
    let failed = loop {
        let mut round_failed = Vec::new();
        for piece in pending.chunks(chunk_size) {
            let batch = run_round(board, &moves, piece, depth, current_color, best_root, &inject, &pool);

            for (idx, res) in batch {
                match res {
                    Ok((mv, score)) => {
                        completed.record(idx, mv, score);
                        if score > best_root {
                            best_root = score;
                        }
                    }
                    Err(()) => round_failed.push(idx),
                }
            }
        }

        if round_failed.is_empty() || round >= MAX_RETRY_ROUNDS {
            break round_failed;
        }
        pending = round_failed;
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
    let e_pool = create_e_core_pool(config.e_core_threads)
        .expect("Failed to create E-core pool");

    /* run both phases at the same time: heavy on p-cores, light on e-cores, so wall-clock
       is max(p, e) instead of p + e. the old serial order existed to hand phase 2 a tightened
       alpha from phase 1, but that handoff is gone (search_moves_parallel goes full-window),
       so serializing just left the e-cores idle through the whole of phase 1. */
    let board_ref: &Board = board;
    let ((phase1_best_move, phase1_best_score), p1_ms,
         (phase2_best_move, phase2_best_score), p2_ms) = std::thread::scope(|s| {
        let heavy = s.spawn(|| {
            let t = Instant::now();
            let r = if !heavy_moves.is_empty() {
                p_pool.install(|| search_moves_parallel(board_ref, &heavy_moves, depth, i32::MIN + 1))
            } else {
                (Move::default(), i32::MIN + 1)
            };
            (r, t.elapsed().as_secs_f64() * 1000.0)
        });
        let light = s.spawn(|| {
            let t = Instant::now();
            let r = if !light_moves.is_empty() {
                e_pool.install(|| search_moves_parallel(board_ref, &light_moves, depth, i32::MIN + 1))
            } else {
                (Move::default(), i32::MIN + 1)
            };
            (r, t.elapsed().as_secs_f64() * 1000.0)
        });
        let (hr, h_ms) = heavy.join().unwrap();
        let (lr, l_ms) = light.join().unwrap();
        (hr, h_ms, lr, l_ms)
    });
    metrics.phase1_time_ms = p1_ms;
    metrics.phase2_time_ms = p2_ms;
    
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

/* search a batch of classified moves in parallel, return best (move, score).
   _alpha used to be the earlier phase's best for cross-phase pruning, but mixing that
   with the full-window per-move search was wrong, so we ignore it now and just search
   each move full-window. keeps the signature for the callers. */
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