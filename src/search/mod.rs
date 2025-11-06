pub mod minimax;
pub mod ordering;
pub mod parallel;
pub mod time_control;
pub mod transposition;
pub mod fault_tolerant;
pub mod recovery;

pub use minimax::{search, alphabeta};
pub use parallel::{parallel_search, parallel_search_with_policy, parallel_search_with_fault};
pub use fault_tolerant::{with_recovery, should_inject_panic};