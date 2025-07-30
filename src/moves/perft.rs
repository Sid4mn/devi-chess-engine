use std::time::Instant;
use crate::board::{Board, BoardRepresentation};
use crate::moves::{generate_legal_moves};

pub fn perft(board: &mut Board, depth: u32) -> u64 {
    if depth == 0 {
        return 1;
    }
    
    let mut nodes = 0;
    let moves = generate_legal_moves(board, board.to_move());
    
    if depth == 1 {
        return moves.len() as u64;
    }
    
    for mv in moves {
        let undo = board.make_move(&mv);
        nodes += perft(board, depth - 1);
        board.unmake_move(&mv, undo);
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

pub fn perft_divide(board: &mut Board, depth: u32) -> (Vec<(String, u64)>, u64) {
    let mut results = Vec::new();
    let mut total = 0;
    
    let moves = generate_legal_moves(board, board.to_move());
    
    for mv in moves {
        let undo = board.make_move(&mv);
        let nodes = if depth == 1 {
            1 
        } else {
            perft(board, depth - 1)
        };
        board.unmake_move(&mv, undo);
        
        results.push((mv.to_algebraic(), nodes));
        total += nodes;
    }
    
    (results, total)
}