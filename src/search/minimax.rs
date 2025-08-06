use crate::board::{Board, BoardRepresentation};
use crate::moves::generate_legal_moves;
use crate::types::*;
use crate::evaluation::evaluate;

pub fn minimax(board: &mut Board, depth: u32, maximizing_player: bool) -> i32 {
    if depth == 0 {
        return evaluate(board);
    }

    let current_color = board.to_move();
    let moves = generate_legal_moves(board, current_color);

    if maximizing_player {
        let mut max_eval = i32::MIN;
        for mv in moves {
            let undo = board.make_move(&mv);
            let eval = minimax(board, depth - 1, false);
            board.unmake_move(&mv, undo);
            max_eval = max_eval.max(eval);
        }
        max_eval
    } else {
        let mut min_eval = i32::MAX;
        for mv in moves {
            let undo = board.make_move(&mv);
            let eval = minimax(board, depth - 1, true);
            board.unmake_move(&mv, undo);
            min_eval = min_eval.min(eval);
        }
        min_eval
    }
}

pub fn alphabeta(board: &mut Board, depth: u32, mut alpha: i32, mut beta: i32, maximizing_player: bool) -> i32 {
    if depth == 0 {
        return evaluate(board);
    }

    let current_color = board.to_move();
    let moves = generate_legal_moves(board, current_color);

    if moves.is_empty() {
        panic!("No legal moves available!");
    }

    if maximizing_player {
        for mv in moves {
            let undo = board.make_move(&mv);
            let eval = alphabeta(board, depth - 1, alpha, beta, false);
            board.unmake_move(&mv, undo);
            alpha = alpha.max(eval);
            if beta <= alpha {
                break;
            }
        }
        alpha
    } else {
        for mv in moves {
            let undo = board.make_move(&mv);
            let eval = alphabeta(board, depth - 1, alpha, beta, true);
            board.unmake_move(&mv, undo);
            beta = beta.min(eval);
            if beta <= alpha {
                break;
            }
        }
        beta
    }
}

pub fn search(board: &mut Board, depth: u32) -> (Move, i32) {
    let current_color = board.to_move();
    let moves = generate_legal_moves(board, current_color);

    if moves.is_empty() {
        panic!("No legal moves available");
    }

    let mut best_move = moves[0];
    let mut best_score = i32::MIN;

    for mv in moves {
        let undo = board.make_move(&mv);
        let score = -alphabeta(board, depth - 1, -i32::MAX, i32::MAX, false);
        board.unmake_move(&mv, undo);
        
        if score > best_score {
            best_score = score;
            best_move = mv;
        }
    }

    (best_move, best_score)
}