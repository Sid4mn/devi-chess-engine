use crate::board::{Board, BoardRepresentation};
use crate::moves::generate_legal_moves;
use crate::scheduling::{create_pool_for_policy, CorePolicy};
use crate::search::minimax::alphabeta;
use crate::search::minimax::MATE_SCORE;
use crate::types::*;
use rayon::prelude::*;

pub fn parallel_search(board: &mut Board, depth: u32) -> (Move, i32) {
    let threads = rayon::current_num_threads();
    parallel_search_with_policy(board, depth, CorePolicy::None, threads, 0.0)
}

/// Root-split parallel search with core policy support
pub fn parallel_search_with_policy(board: &mut Board, depth: u32, policy: CorePolicy, threads: usize, mixed_ratio: f32) -> (Move, i32) {
    let current_color = board.to_move();
    let moves = generate_legal_moves(board, current_color);

    if moves.is_empty() {
        let dummy_move = Move::new(Square(0), Square(0), None, None);
        let score = if board.is_in_check(current_color) {
            -MATE_SCORE // We're checkmated
        } else {
            0 // Stalemate
        };
        return (dummy_move, score);
    }
    let pool = create_pool_for_policy(policy, threads, mixed_ratio);

    pool.install(|| {
        moves
            .par_iter()
            .map(|mv| {
                let mut local_board = board.clone();
                let undo = local_board.make_move(mv);

                let score = -alphabeta(&mut local_board, depth.saturating_sub(1), i32::MIN + 1, i32::MAX - 1,false);
                local_board.unmake_move(mv, undo);

                (*mv, score)
            })
            .max_by_key(|&(_, score)| score)
            .expect("iterator is non-empty")
    })
}
