use crate::scheduling::CorePolicy;
use clap::Parser;

#[derive(Parser)]
pub struct Cli {
    #[arg(long, default_value_t = 1)]
    pub threads: usize,

    #[arg(long, default_value_t = 4)]
    pub depth: u32,

    #[arg(long, default_value_t = 5)]
    pub warmup: usize,

    #[arg(long, default_value_t = 10)]
    pub runs: usize,

    #[arg(long)]
    pub benchmark: bool,

    #[arg(long)]
    pub soak: bool,

    #[arg(long)]
    pub perft: bool,

    #[arg(long, help = "Use parallel perf computation", default_value_t = false)]
    pub parallel_perft: bool,

    #[arg(long)]
    pub perft_divide: bool,

    #[arg(long)]
    pub inject_panic: Option<usize>,

    #[arg(long)]
    pub dump_crashes: bool,

    #[arg(
        long,
        value_enum,
        help = "Core scheduling policy for heterogeneous architectures"
    )]
    pub core_policy: Option<CorePolicy>,

    #[arg(
        long,
        default_value_t = 0.75,
        help = "Ratio of fast cores in mixed mode (0.0-1.0)"
    )]
    pub mixed_ratio: f32,
}

pub fn parse_args() -> Cli {
    Cli::parse()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn parse_test_args(args: &[&str]) -> Cli {
        Cli::try_parse_from(args).expect("Failed to parse test args")
    }

    #[test]
    fn test_default_threads() {
        let args = parse_test_args(&["devi"]);
        assert_eq!(args.threads, 1);
    }

    #[test]
    fn test_custom_threads() {
        let args = parse_test_args(&["devi", "--threads", "8"]);
        assert_eq!(args.threads, 8);
    }

    #[test]
    fn test_benchmark_flag() {
        let args = parse_test_args(&["devi", "--benchmark"]);
        assert_eq!(args.benchmark, true);
    }

    #[test]
    fn test_soak_flag() {
        let args = parse_test_args(&["devi", "--soak", "--threads", "8", "--runs", "100"]);
        assert_eq!(args.soak, true);
        assert_eq!(args.threads, 8);
        assert_eq!(args.runs, 100);
    }

    #[test]
    fn test_core_policy() {
        let args = parse_test_args(&["devi", "--core-policy", "fast"]);
        assert!(args.core_policy.is_some());
        assert!(matches!(args.core_policy, Some(CorePolicy::FastBias)));
    }

    #[test]
    fn test_mixed_ratio() {
        let args = parse_test_args(&["devi", "--mixed-ratio", "0.5"]);
        assert_eq!(args.mixed_ratio, 0.5);
    }
}
