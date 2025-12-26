use crate::board::{Board, BoardRepresentation};
use crate::moves::generate_legal_moves;
use crate::types::{Color, SpecialMove};
use rayon::prelude::*;
use std::fmt;
use std::time::Instant;

/// Detailed perft statistics matching Chess Programming Wiki format
#[derive(Debug, Clone, Copy, Default)]
pub struct PerftStats {
    pub nodes: u64,
    pub captures: u64,
    pub en_passant: u64,
    pub castles: u64,
    pub promotions: u64,
    pub checks: u64,
    pub discovery_checks: u64,
    pub double_checks: u64,
    pub checkmates: u64,
}

impl PerftStats {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Add another PerftStats to this one
    pub fn add(&mut self, other: &PerftStats) {
        self.nodes += other.nodes;
        self.captures += other.captures;
        self.en_passant += other.en_passant;
        self.castles += other.castles;
        self.promotions += other.promotions;
        self.checks += other.checks;
        self.discovery_checks += other.discovery_checks;
        self.double_checks += other.double_checks;
        self.checkmates += other.checkmates;
    }
}

impl fmt::Display for PerftStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Nodes: {}, Captures: {}, E.p.: {}, Castles: {}, Promotions: {}, Checks: {}, Checkmates: {}",
            self.nodes, self.captures, self.en_passant, self.castles, 
            self.promotions, self.checks, self.checkmates
        )
    }
}

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

/// Detailed perft that counts captures, en passant, castles, promotions, checks, and checkmates
pub fn perft_detailed(board: &mut Board, depth: u32) -> PerftStats {
    let mut stats = PerftStats::new();
    
    if depth == 0 {
        stats.nodes = 1;
        return stats;
    }

    let current_color = board.to_move();
    let opponent_color = match current_color {
        Color::White => Color::Black,
        Color::Black => Color::White,
    };
    let moves = generate_legal_moves(board, current_color);

    for mv in &moves {
        let is_capture = board.get_piece(mv.to).is_some() 
            || mv.special_move == Some(SpecialMove::EnPassant);
        let is_en_passant = mv.special_move == Some(SpecialMove::EnPassant);
        let is_castle = mv.special_move == Some(SpecialMove::Castle);
        let is_promotion = mv.promotion.is_some();
        
        let undo = board.make_move(mv);
        
        if depth == 1 {
            stats.nodes += 1;
            if is_capture { stats.captures += 1; }
            if is_en_passant { stats.en_passant += 1; }
            if is_castle { stats.castles += 1; }
            if is_promotion { stats.promotions += 1; }
            
            if let Some(king_sq) = board.find_king(opponent_color) {
                let attacker_count = board.count_attackers(king_sq, current_color);
                if attacker_count > 0 {
                    stats.checks += 1;
                    if attacker_count >= 2 {
                        stats.double_checks += 1;
                    }
                    let opponent_moves = generate_legal_moves(board, opponent_color);
                    if opponent_moves.is_empty() {
                        stats.checkmates += 1;
                    }
                }
            }
        } else {
            let child_stats = perft_detailed(board, depth - 1);
            stats.add(&child_stats);
        }
        
        board.unmake_move(mv, undo);
    }

    stats
}

/// Parallel version of perft_detailed for faster deep analysis
pub fn perft_detailed_parallel(board: &mut Board, depth: u32) -> PerftStats {
    if depth == 0 {
        let mut stats = PerftStats::new();
        stats.nodes = 1;
        return stats;
    }

    if depth <= 3 {
        return perft_detailed(board, depth);
    }

    let current_color = board.to_move();
    let moves = generate_legal_moves(board, current_color);

    moves
        .par_iter()
        .map(|mv| {
            let mut local_board = board.clone();
            let undo = local_board.make_move(mv);
            let child_stats = perft_detailed(&mut local_board, depth - 1);
            local_board.unmake_move(mv, undo);
            child_stats
        })
        .reduce(PerftStats::new, |mut acc, s| {
            acc.add(&s);
            acc
        })
}

/// Perft with detailed output for verification - prints all statistics
pub fn perft_verify(board: &mut Board, depth: u32) -> PerftStats {
    println!("Perft verification with detailed statistics:");
    println!("Depth\tNodes\t\tCaptures\tE.p.\tCastles\tPromos\tChecks\tCheckmates");
    println!("-----\t-----\t\t--------\t----\t-------\t------\t------\t----------");
    
    for d in 0..=depth {
        let stats = perft_detailed(board, d);
        println!(
            "{}\t{}\t\t{}\t\t{}\t{}\t{}\t{}\t{}",
            d, stats.nodes, stats.captures, stats.en_passant,
            stats.castles, stats.promotions, stats.checks, stats.checkmates
        );
    }
    
    perft_detailed(board, depth)
}

/// Multi-position perft verification from a list of FEN positions
/// Returns true if all positions pass, false otherwise
pub fn verify_positions(positions: &[(& str, u32, u64)]) -> bool {
    let mut all_passed = true;
    
    println!("Verifying {} positions...", positions.len());
    println!("------------------------------------------------------------");
    
    for (i, (fen, depth, expected)) in positions.iter().enumerate() {
        let mut board = match Board::from_fen(fen) {
            Ok(b) => b,
            Err(e) => {
                println!("Position {}: Failed to parse FEN: {}", i + 1, e);
                all_passed = false;
                continue;
            }
        };
        
        let actual = perft(&mut board, *depth);
        
        if actual == *expected {
            println!("[OK] Position {}: depth {} = {} correct", i + 1, depth, actual);
        } else {
            println!("[FAIL] Position {}: depth {} = {} (expected {})", 
                     i + 1, depth, actual, expected);
            all_passed = false;
        }
    }
    
    println!("------------------------------------------------------------");
    if all_passed {
        println!("All {} positions passed!", positions.len());
    } else {
        println!("Some positions FAILED!");
    }
    
    all_passed
}

/// Multi-position detailed verification with statistics
pub fn verify_positions_detailed(positions: &[(&str, u32, PerftStats)]) -> bool {
    let mut all_passed = true;
    
    println!("Verifying {} positions with detailed statistics...", positions.len());
    
    for (i, (fen, depth, expected)) in positions.iter().enumerate() {
        let mut board = match Board::from_fen(fen) {
            Ok(b) => b,
            Err(e) => {
                println!("[FAIL] Position {}: Failed to parse FEN: {}", i + 1, e);
                all_passed = false;
                continue;
            }
        };
        
        let actual = perft_detailed(&mut board, *depth);
        
        let mut position_passed = true;
        let mut mismatches = Vec::new();
        
        if actual.nodes != expected.nodes {
            mismatches.push(format!("nodes: {} vs {}", actual.nodes, expected.nodes));
            position_passed = false;
        }
        if actual.captures != expected.captures {
            mismatches.push(format!("captures: {} vs {}", actual.captures, expected.captures));
            position_passed = false;
        }
        if actual.en_passant != expected.en_passant {
            mismatches.push(format!("e.p.: {} vs {}", actual.en_passant, expected.en_passant));
            position_passed = false;
        }
        if actual.castles != expected.castles {
            mismatches.push(format!("castles: {} vs {}", actual.castles, expected.castles));
            position_passed = false;
        }
        if actual.promotions != expected.promotions {
            mismatches.push(format!("promos: {} vs {}", actual.promotions, expected.promotions));
            position_passed = false;
        }
        if actual.checks != expected.checks {
            mismatches.push(format!("checks: {} vs {}", actual.checks, expected.checks));
            position_passed = false;
        }
        if actual.checkmates != expected.checkmates {
            mismatches.push(format!("checkmates: {} vs {}", actual.checkmates, expected.checkmates));
            position_passed = false;
        }
        
        if position_passed {
            println!("[OK] Position {}: depth {} - all stats match", i + 1, depth);
        } else {
            println!("[FAIL] Position {}: depth {} - mismatches: {}", 
                     i + 1, depth, mismatches.join(", "));
            all_passed = false;
        }
    }
    
    all_passed
}

pub fn perft_parallel(board: &mut Board, depth: u32) -> u64 {
    if depth == 0 {
        return 1;
    }

    // Use serial for shallow depths (overhead not worth it)
    if depth <= 3 {
        return perft(board, depth);
    }

    let current_color = board.to_move();
    let moves = generate_legal_moves(board, current_color);

    moves
        .par_iter()
        .map(|mv| {
            let mut local_board = board.clone();
            let undo = local_board.make_move(mv);
            let nodes = perft(&mut local_board, depth - 1);
            local_board.unmake_move(mv, undo);
            nodes
        })
        .sum()
}

pub fn benchmark_perft(board: &mut Board) {
    for depth in 1..=3 {
        let start = Instant::now();
        let nodes = perft(board, depth);
        let elapsed = start.elapsed();

        let nodes_per_sec: u128 = (nodes as u128 * 1000) / elapsed.as_millis() as u128;
        println!("Nodes_per_sec = {:}", nodes_per_sec);
    }
}

pub fn perft_divide(board: &mut Board, depth: u32) -> (Vec<(String, u64)>, u64) {
    let mut results = Vec::new();
    let mut total = 0;

    let moves = generate_legal_moves(board, board.to_move());

    for mv in moves {
        let undo = board.make_move(&mv);
        let nodes = perft(board, depth - 1);
        board.unmake_move(&mv, undo);

        results.push((mv.to_algebraic(), nodes));
        total += nodes;
    }

    (results, total)
}
