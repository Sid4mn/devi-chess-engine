pub mod minimax;
pub use minimax::{search, alphabeta, minimax};
pub mod parallel;
pub use parallel::parallel_search;