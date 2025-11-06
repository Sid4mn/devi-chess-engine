use crate::board::{Board, BoardRepresentation};
use crate::moves::generate_legal_moves;
use crate::scheduling::{create_pool_for_policy, CorePolicy};
use crate::search::fault_tolerant::should_inject_panic;
use crate::search::minimax::alphabeta;
use crate::search::minimax::MATE_SCORE;
use crate::types::*;
use rayon::prelude::*;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

static MOVE_COUNTER: AtomicUsize = AtomicUsize::new(0);
static SHOULD_PANIC: AtomicBool = AtomicBool::new(false);

pub fn parallel_search(board: &mut Board, depth: u32) -> (Move, i32) {
    let threads = rayon::current_num_threads();
    parallel_search_with_policy(board, depth, CorePolicy::None, threads, 0.0)
}

pub fn parallel_search_with_policy(
    board: &mut Board,
    depth: u32,
    policy: CorePolicy,
    threads: usize,
    mixed_ratio: f32,
) -> (Move, i32) {
    parallel_search_with_fault(board, depth, policy, threads, mixed_ratio, None)
}

pub fn parallel_search_with_fault(
    board: &mut Board,
    depth: u32,
    policy: CorePolicy,
    threads: usize,
    mixed_ratio: f32,
    inject_panic_at: Option<usize>,
) -> (Move, i32) {
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
