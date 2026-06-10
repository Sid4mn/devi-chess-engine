/*! Root move ordering. Alpha-beta only prunes well when a good move is searched first
    (a tight alpha lets the rest cut off). We have no root transposition/history table
    yet, so we seed the parallel search with the best move from a cheap shallow search.
    Ordering only affects SPEED — the result is identical regardless of order. */

use crate::board::{Board, BoardRepresentation};
use crate::evaluation::evaluate;
use crate::search::minimax::root_move_score_bounded;
use crate::types::{Color, Move};

/// Static eval of the position `mv` leads to, from `root_color`'s POV (evaluate is
/// white-relative, so flip for black).
fn one_ply_score(board: &mut Board, mv: &Move, root_color: Color) -> i32 {
    let undo = board.make_move(mv);
    let abs = evaluate(board);
    board.unmake_move(mv, undo);
    if root_color == Color::White {
        abs
    } else {
        -abs
    }
}

/// Index of the move with the best 1-ply static eval. Cheap but weak (ignores tactics),
/// so it's only a fallback. Ties resolve to the lowest index.
pub fn best_root_move_index(board: &mut Board, moves: &[Move], root_color: Color) -> usize {
    let mut best_idx = 0;
    let mut best_score = i32::MIN;
    for (i, mv) in moves.iter().enumerate() {
        let s = one_ply_score(board, mv, root_color);
        if s > best_score {
            best_score = s;
            best_idx = i;
        }
    }
    best_idx
}

/// Index of the seed move chosen by a cheap shallow search: best-at-shallow is almost
/// always best-at-full, giving a tight alpha for the parallel search to prune against.
/// Sequential and self-pruning, so it stays cheap. Ties resolve to the lowest index.
pub fn best_root_move_index_shallow(
    board: &mut Board,
    moves: &[Move],
    root_color: Color,
    shallow_depth: u32,
) -> usize {
    top_two_root_moves_shallow(board, moves, root_color, shallow_depth).0
}

/// Same shallow search, but also keep the runner-up (None with a single legal move).
/// The runner-up is the fault-recovery seed: if the best move's deep search dies, a
/// near-best bound still prunes well - an arbitrary replacement move wouldn't.
pub fn top_two_root_moves_shallow(
    board: &mut Board,
    moves: &[Move],
    root_color: Color,
    shallow_depth: u32,
) -> (usize, Option<usize>) {
    let mut best_idx = 0;
    let mut best_score = i32::MIN + 1; // "no bound yet"
    let mut second_idx: Option<usize> = None;
    let mut second_score = i32::MIN + 1;
    for (i, mv) in moves.iter().enumerate() {
        let score = root_move_score_bounded(board, mv, shallow_depth, root_color, best_score);
        if score > best_score {
            // demote the old best to runner-up (unless it was just the placeholder)
            if i > 0 {
                second_score = best_score;
                second_idx = Some(best_idx);
            }
            best_score = score;
            best_idx = i;
        } else if score > second_score || second_idx.is_none() {
            second_score = score;
            second_idx = Some(i);
        }
    }
    (best_idx, second_idx)
}
