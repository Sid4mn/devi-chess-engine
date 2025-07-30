use crate::moves::generate_moves;
use crate::types::*;
use crate::board::{Board, BoardRepresentation};

pub fn generate_legal_moves(board: &mut Board, color: Color) -> Vec<Move> {
    let mut legal_moves = Vec::new();
    let pseudo_moves: Vec<Move> = generate_moves(board, color);

    for _move in pseudo_moves {
        let undo_info = board.make_move(&_move);
        if !board.is_in_check(color) {
            legal_moves.push(_move);
        }
        board.unmake_move(&_move, undo_info);
    }
    legal_moves
}