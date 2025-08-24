pub mod minimax;
pub use minimax::{alphabeta, minimax, search, INFINITY, MATE_SCORE};
pub mod parallel;
pub use parallel::parallel_search;
pub mod fault_tolerant;
pub use fault_tolerant::{search_root_fault};
