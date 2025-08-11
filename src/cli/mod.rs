pub mod cli;
pub mod commands;

pub use cli::{Cli, parse_args};
pub use commands::{
    run_full_benchmark,
    run_single_search, 
    run_soak_test,
    run_perft_test,
};