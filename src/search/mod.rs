pub mod fault_tolerant;
pub mod minimax;
pub mod ordering;
pub mod parallel;
pub mod probe;
pub mod recovery;
pub mod time_control;
pub mod transposition;

pub use fault_tolerant::with_recovery;
pub use minimax::{alphabeta, root_move_score, root_move_score_bounded, search};
pub use ordering::{best_root_move_index, best_root_move_index_shallow, top_two_root_moves_shallow};
pub use parallel::{parallel_search, parallel_search_with_fault, parallel_search_with_fault_set, parallel_search_with_policy, two_phase_search, InjectionPlan, SearchOutcome, TwoPhaseConfig, TwoPhaseMetrics, two_phase_search_with_metrics};
