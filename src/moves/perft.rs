use crate::board::{Board, BoardRepresentation};
use crate::moves::{generate_legal_moves};

 pub fn perft(board: &mut Board, depth:u32) -> u64 {
    if depth == 0 {
        return 1
    }

    let mut nodes = 0;
    let color_to_move = board.to_move();
    let legal_moves = generate_legal_moves(board, color_to_move);

    for _mv in legal_moves.iter() {
        let undo = board.make_move(_mv);
        nodes += perft(board, depth-1);
        board.unmake_move(_mv, undo);
    }

    nodes

 }