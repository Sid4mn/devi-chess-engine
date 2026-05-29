use crate::board::{Board, BoardRepresentation};
use crate::evaluation::evaluate;
use crate::moves::generate_legal_moves;
use crate::types::*;

pub const INFINITY: i32 = 1_000_000;
pub const MATE_SCORE: i32 = 100_000; // high but not Max to allow mate in N moves scoring

// Non pruning basic minimax.
// eval is white-relative so this isn't negamax - maximizing_player just says
// whose turn it is. don't negate the result at the root (see root_move_score).
pub fn minimax(board: &mut Board, depth: u32, maximizing_player: bool) -> i32 {
    if depth == 0 {
        return evaluate(board); // leaf
    }

    let current_color = board.to_move();
    let moves = generate_legal_moves(board, current_color);

    if moves.is_empty() {
        if board.is_in_check(current_color) {
            // Checkmate - return negative score for losing side
            return if maximizing_player {
                -MATE_SCORE
            } else {
                MATE_SCORE
            };
        } else {
            // Stalemate - draw
            return 0;
        }
    }

    if maximizing_player {
        let mut max_eval = i32::MIN;
        for mv in moves {
            let undo = board.make_move(&mv);
            let eval = minimax(board, depth - 1, false); // Opp wants min
            board.unmake_move(&mv, undo);
            max_eval = max_eval.max(eval); // keep best score
        }
        max_eval
    } else {
        let mut min_eval = i32::MAX;
        for mv in moves {
            let undo = board.make_move(&mv);
            let eval = minimax(board, depth - 1, true); // my turn, want max
            board.unmake_move(&mv, undo);
            min_eval = min_eval.min(eval); // keep worst score
        }
        min_eval
    }
}

// alpha-beta. absolute (white-relative) eval, NOT negamax.
// maximizing_player = white to move at this node. ply tracks depth from root,
// only used so we can score mates by distance.
pub fn alphabeta(
    board: &mut Board,
    depth: u32,
    mut alpha: i32,
    mut beta: i32,
    maximizing_player: bool,
    ply: u32,
) -> i32 {
    if depth == 0 {
        return evaluate(board);
    }

    let current_color = board.to_move();
    let moves = generate_legal_moves(board, current_color);

    if moves.is_empty() {
        if board.is_in_check(current_color) {
            // subtract ply so closer mates score higher
            let mate = MATE_SCORE - ply as i32;
            return if maximizing_player { -mate } else { mate };
        } else {
            //Stalemate - draw
            return 0;
        }
    }

    if maximizing_player {
        for mv in moves {
            let undo = board.make_move(&mv);
            let eval = alphabeta(board, depth - 1, alpha, beta, false, ply + 1);
            board.unmake_move(&mv, undo);
            alpha = alpha.max(eval);
            if beta <= alpha {
                break; // stop searching, prune, opp won't let us reach here.
            }
        }
        alpha
    } else {
        for mv in moves {
            let undo = board.make_move(&mv);
            let eval = alphabeta(board, depth - 1, alpha, beta, true, ply + 1);
            board.unmake_move(&mv, undo);
            beta = beta.min(eval);
            if beta <= alpha {
                break; // stop searching, prune, we won't let opp reach here.
            }
        }
        beta
    }
}

// score one root move from root_color's point of view (higher = better for us).
// make the move, search child with the correct maximizing flag, no negation,
// then flip sign for black so callers can just take the max for either color.
pub fn root_move_score(board: &mut Board, mv: &Move, depth: u32, root_color: Color) -> i32 {
    let undo = board.make_move(mv);
    // opponent to move now, so maximize only if they're white
    let child_is_white = board.to_move() == Color::White;
    let abs = alphabeta(
        board,
        depth.saturating_sub(1),
        i32::MIN + 1,
        i32::MAX - 1,
        child_is_white,
        1,
    );
    board.unmake_move(mv, undo);

    if root_color == Color::White {
        abs
    } else {
        -abs // flip white-relative score to black's POV
    }
}

pub fn search(board: &mut Board, depth: u32) -> (Move, i32) {
    let current_color = board.to_move();
    let moves = generate_legal_moves(board, current_color);

    if moves.is_empty() {
        let dummy_move = Move::new(Square(0), Square(0), None, None);
        let score = if board.is_in_check(current_color) {
            -MATE_SCORE //We're checkmated
        } else {
            0 //Stalemate
        };
        return (dummy_move, score);
    }

    let mut best_move = moves[0];
    let mut best_score = i32::MIN;

    for mv in moves {
        // score is side-to-move relative, plain max works for both colors
        let score = root_move_score(board, &mv, depth, current_color);

        if score > best_score {
            best_score = score;
            best_move = mv;
        }
    }

    (best_move, best_score)
}
