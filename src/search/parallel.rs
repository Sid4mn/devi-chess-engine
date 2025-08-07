use crate::board::{Board, BoardRepresentation};
use crate::moves::generate_legal_moves;
use crate::search::minimax::alphabeta;
use crate::types::*;
use rayon::prelude::*;

pub fn parallel_search(board: &mut Board, depth: u32) -> (Move, i32) {
    let current_color = board.to_move();
    let moves = generate_legal_moves(board, current_color);

    assert!(
        !moves.is_empty(),
        "No legal moves available in position {:?}",board
    );

    moves.par_iter()
        .map(|mv|{
            let mut local_board = board.clone();
            let undo = local_board.make_move(mv);

            let score = -alphabeta(&mut local_board, depth.saturating_sub(1), i32::MIN + 1, i32::MAX - 1, false);
            local_board.unmake_move(mv, undo);

            (*mv, score)
        }).max_by_key(|&(_, score)| score)
        .expect("iterator is non-empty")
    }