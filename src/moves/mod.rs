use crate::types::*;
use crate::board::Board;

pub mod movegen;
pub mod piece_moves;

pub use movegen::generate_moves;
pub use piece_moves::generate_pawn_moves;