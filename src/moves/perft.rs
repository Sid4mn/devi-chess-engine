use std::time::Instant;
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

pub fn benchmark_perft(_board: &mut Board) {
    for depth in 1..=3 {
        let start = Instant::now();
        let nodes = perft(_board, depth);
        let elapsed = start.elapsed();

        let nodes_per_sec: u128 = (nodes as u128 * 1000 )/ elapsed.as_millis() as u128;
        println!("Nodes_per_sec = {:}", nodes_per_sec);
    }
}

pub fn perft_divide(board: &mut Board, depth: u32) -> u64 {
    if depth == 0 {
        return 1
    }

    let color = board.to_move();
    let moves = generate_legal_moves(board, color);
    let mut total = 0;

    for mv in &moves {
        let undo = board.make_move(mv);
        let subtree = perft(board, depth - 1);
        board.unmake_move(mv, undo);

        println!("{}: {}", mv.to_algebraic(), subtree);
        total += subtree;
    }

    print!("Total: {}", total);
    total
}