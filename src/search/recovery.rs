use crate::board::Board;

// Simplified - checkpointing disabled for now
pub struct CheckpointManager {
    pub interval: usize,
}

impl CheckpointManager {
    pub fn new(interval: usize) -> Self {
        Self { interval }
    }
    
    // Stub methods - not used in simplified recovery
    pub fn save(&self, _worker_id: usize, _checkpoint: SearchCheckpoint) {}
    pub fn get(&self, _worker_id: usize) -> Option<SearchCheckpoint> { None }
    pub fn clear(&self, _worker_id: usize) {}
}

#[derive(Clone)]
pub struct SearchCheckpoint {
    pub board: Board,
    pub depth_remaining: u8,
    pub alpha: i32,
    pub beta: i32,
    pub move_index: usize,
    pub nodes_searched: u64,
    pub best_score: i32,
}