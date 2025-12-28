use crate::board::{Board, BoardRepresentation};
use crate::moves::generate_legal_moves;
use crate::types::{ClassifiedMove, Move, MovePhase};

/// Configuration for move classification heuristics
#[derive(Clone, Copy, Debug)]
pub struct ClassificationConfig {
    /// Ratio of moves to classify as "heavy" (0.0-1.0, default 0.6 = top 60%)
    pub heavy_ratio: f32,
    /// If light moves exceed this % of total nodes, move all to P-cores (default 0.3)
    pub light_threshold: f32,
}

impl Default for ClassificationConfig {
    fn default() -> Self {
        Self {
            heavy_ratio: 0.6,
            light_threshold: 0.3,
        }
    }
}

/// Count nodes in a subtree at given depth (no alpha-beta, just node counting)
pub fn probe_move(board: &Board, mv: &Move, depth: u8) -> u64 {
    if depth == 0 {
        return 1;
    }
    
    let mut temp_board = board.clone();
    let undo = temp_board.make_move(mv);
    
    let current_color = temp_board.to_move();
    let moves = generate_legal_moves(&mut temp_board, current_color);
    
    if moves.is_empty() {
        temp_board.unmake_move(mv, undo);
        return 1;
    }
    
    let mut node_count = 0u64;
    for child_mv in &moves {
        node_count += probe_move(&temp_board, child_mv, depth - 1);
    }
    
    temp_board.unmake_move(mv, undo);
    node_count.max(1)
}

/// Probe all root moves and return (move, node_count) pairs
pub fn probe_root_moves(board: &Board, moves: &[Move], depth: u8) -> Vec<(Move, u64)> {
    moves
        .iter()
        .map(|mv| {
            let node_count = probe_move(board, mv, depth);
            (*mv, node_count)
        })
        .collect()
}

pub fn classify_moves(probed: Vec<(Move, u64)>) -> (Vec<ClassifiedMove>, Vec<ClassifiedMove>) {
    classify_moves_with_config(probed, &ClassificationConfig::default())
}

pub fn classify_moves_with_config(probed: Vec<(Move, u64)>, config: &ClassificationConfig) -> (Vec<ClassifiedMove>, Vec<ClassifiedMove>) {
    if probed.is_empty() {
        return (vec![], vec![]);
    }
    
    let n = probed.len();
    
    // Special case: single move goes to heavy (P-cores)
    if n == 1 {
        let (mv, nodes) = probed[0];
        return (
            vec![ClassifiedMove {
                mv,
                subtree_nodes: nodes,
                phase: MovePhase::Heavy,
            }],
            vec![],
        );
    }
    
    let mut sorted_counts: Vec<u64> = probed.iter().map(|(_, n)| *n).collect();
    sorted_counts.sort_unstable_by(|a, b| b.cmp(a));  // Descending
    
    let heavy_count = ((n as f32) * config.heavy_ratio) as usize;
    let heavy_count = heavy_count.max(1);
    let threshold = sorted_counts.get(heavy_count - 1).copied().unwrap_or(0);
    
    let mut heavy = Vec::new();
    let mut light = Vec::new();
    let mut heavy_assigned = 0;
    
    for (mv, nodes) in probed.iter() {
        let is_heavy = *nodes >= threshold && heavy_assigned < heavy_count;
        
        let classified = ClassifiedMove {
            mv: *mv,
            subtree_nodes: *nodes,
            phase: if is_heavy { MovePhase::Heavy } else { MovePhase::Light },
        };
        
        if is_heavy {
            heavy.push(classified);
            heavy_assigned += 1;
        } else {
            light.push(classified);
        }
    }
    
    let total_nodes: u64 = heavy.iter().chain(light.iter()).map(|cm| cm.subtree_nodes).sum();
    let light_nodes: u64 = light.iter().map(|cm| cm.subtree_nodes).sum();
    let light_pct = if total_nodes > 0 { light_nodes as f32 / total_nodes as f32 } else { 0.0 };
    
    if !light.is_empty() && light_pct > config.light_threshold {
        for mut cm in light.drain(..) {
            cm.phase = MovePhase::Heavy;
            heavy.push(cm);
        }
    }
    
    (heavy, light)
}
