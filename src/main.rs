use devi::cli;

fn main() {
    println!("devi Chess Engine v{}", env!("CARGO_PKG_VERSION"));
    println!("------------------------");

    let args = cli::parse_args();
    run_command(&args);
}

fn run_command(args: &cli::Cli) {
    match (args.benchmark, args.soak, args.perft, args.inject_panic.is_some()) {
        (true, _, _, _) => cli::run_full_benchmark(&args),
        (_, true, _, _) => cli::run_soak_test(&args),
        (_, _, true, _) => cli::run_perft_test(&args),
        (_, _, _, true) if args.dump_crashes => cli::run_fault_analysis(&args),
        _ => cli::run_single_search(args),
    }
}
