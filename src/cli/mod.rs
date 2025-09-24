pub mod cli;
pub mod commands;

pub use cli::{parse_args, Cli};
pub use commands::{
    run_fault_analysis, run_full_benchmark, run_perft_test, run_single_search, run_soak_test,
};
