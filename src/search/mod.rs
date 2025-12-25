pub mod fault_tolerant;
pub mod minimax;
pub mod ordering;
pub mod parallel;
pub mod probe;
pub mod recovery;
pub mod time_control;
pub mod transposition;

pub use fault_tolerant::{should_inject_panic, with_recovery};
pub use minimax::{alphabeta, search};
pub use parallel::{parallel_search, parallel_search_with_fault, parallel_search_with_policy, two_phase_search, TwoPhaseConfig};
