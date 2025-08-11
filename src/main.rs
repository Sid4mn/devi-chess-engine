use devi::cli;

fn main() {
    println!("devi Chess Engine v0.2.0");
    println!("------------------------");

    let args = cli::parse_args();
    run_command(&args);
}

fn run_command(args: &cli::Cli) {
    match(args.benchmark, args.soak, args.perft) {
        (true, _, _) => cli::run_full_benchmark(&args),
        (_, true, _) => cli::run_soak_test(&args),
        (_, _, true) => cli::run_perft_test(&args),
        _ => cli::run_single_search(args),
    }
}
