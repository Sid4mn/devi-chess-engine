use devi::cli;

fn main() {
    println!("devi Chess Engine v{}", env!("CARGO_PKG_VERSION"));
    println!("------------------------");

    let args = cli::parse_args();
    run_command(&args);
}

fn run_command(args: &cli::Cli) {
    if args.thread_recovery {
        cli::commands::run_recovery_analysis(&args);
        return;
    }

    if args.fault_analysis {
        cli::commands::run_fault_overhead_analysis(&args);
        return;
    }

    if args.inject_panic.is_some() {
        if args.benchmark {
            // Run benchmark WITH fault injection
            cli::run_full_benchmark(&args);
        } else {
            // Regular search with fault injection
            cli::run_single_search(&args);
        }
    } else {
        // Normal dispatch without fault injection
        match (args.benchmark, args.soak, args.perft) {
            (true, _, _) => cli::run_full_benchmark(&args),
            (_, true, _) => cli::run_soak_test(&args),
            (_, _, true) => cli::run_perft_test(&args),
            _ => cli::run_single_search(&args),
        }
    }
}
