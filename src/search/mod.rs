pub mod minimax;
pub use minimax::{search, alphabeta, minimax, MATE_SCORE, INFINITY};
pub mod parallel;
pub use parallel::parallel_search;