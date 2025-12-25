use crate::scheduling::CorePolicy;
use clap::Parser;

#[derive(Parser)]
pub struct Cli {
    // EXECUTION
    #[arg(long)]
    pub benchmark: bool,

    #[arg(long, help = "Run full thread count sweep for benchmarking")]
    pub benchmark_sweep: bool,

    #[arg(long)]
    pub soak: bool,

    // ENGINE DEBUG
    #[arg(long)]
    pub perft: bool,

    #[arg(long, help = "Use parallel perf computation", default_value_t = false)]
    pub parallel_perft: bool,

    #[arg(long)]
    pub perft_divide: bool,

    // CORE PARAMETERS
    #[arg(long, default_value_t = 1)]
    pub threads: usize,

    #[arg(long, default_value_t = 4)]
    pub depth: u32,

    #[arg(long, default_value_t = 5)]
    pub warmup: usize,

    #[arg(long, default_value_t = 10)]
    pub runs: usize,

    // HETEROGENEOUS SCHEDULING
    #[arg(long, value_enum, help = "Core scheduling policy for heterogeneous architectures")]
    pub core_policy: Option<CorePolicy>,

    #[arg(long, default_value_t = 0.80, help = "Ratio of fast cores in mixed mode (0.0-1.0)")]
    pub mixed_ratio: f32,

    // FAULT TOLERANCE
    #[arg(long)]
    pub inject_panic: Option<usize>,

    #[arg(long, help = "Enable thread recovery with checkpointing")]
    pub thread_recovery: bool,

    #[arg(long, help = "Run comprehensive fault tolerance overhead analysis")]
    pub fault_analysis: bool,

    #[arg(long, help = "Custom CSV output path (default: benchmarks/speedup.csv)")]
    pub csv_output: Option<String>,

    /// Enable two-phase heterogeneous scheduling
    #[arg(long, default_value_t = false)]
    pub two_phase: bool,

    /// Probe depth for move classification (1-2 recommended)
    #[arg(long, default_value_t = 1)]
    pub probe_depth: u8,

    /// Number of P-core threads for Phase 1
    #[arg(long, default_value_t = 8)]
    pub p_cores: usize,

    /// Number of E-core threads for Phase 2
    #[arg(long, default_value_t = 2)]
    pub e_cores: usize,
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
    fn test_boolean_flags() {
        let args = parse_test_args(&["devi", "--benchmark-sweep", "--soak", "--benchmark"]);
        assert_eq!(args.soak, true);
        assert_eq!(args.benchmark, true);
        assert_eq!(args.benchmark_sweep, true);
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

    #[test]
    fn test_csv_output_flag() {
        let args = parse_test_args(&["devi", "--csv-output", "custom/path.csv"]);
        assert_eq!(args.csv_output, Some("custom/path.csv".to_string()));
    }

    #[test]
    fn test_csv_output_default() {
        let args = parse_test_args(&["devi"]);
        assert_eq!(args.csv_output, None);
    }
}
