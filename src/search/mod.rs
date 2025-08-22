pub mod minimax;
pub use minimax::{alphabeta, minimax, search, INFINITY, MATE_SCORE};
pub mod parallel;
pub use parallel::parallel_search;
