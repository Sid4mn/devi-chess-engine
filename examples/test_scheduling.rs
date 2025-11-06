use devi::benchmark::statistics::BenchmarkStats;
use devi::benchmark::timer::PrecisionTimer;
use devi::scheduling::{create_pool_for_policy, CorePolicy};
use rayon::prelude::*;

pub fn count_primes(limit: u64) -> u64 {
    let mut count = 0;
    for n in 2..limit {
        let mut is_prime = true;
        let sqrt_n = (n as f64).sqrt() as u64 + 1;
        for i in 2..sqrt_n {
            if n % i == 0 {
                is_prime = false;
                break;
            }
        }
        if is_prime {
            count += 1;
        }
    }
    count
}

pub fn test_single_policy(
    policy: CorePolicy,
    num_threads: usize,
    mixed_ratio: f32,
) -> BenchmarkStats {
    println!("\nTesting Policy: {:?}", policy);
    if matches!(policy, CorePolicy::Mixed) {
        println!(
            "  Mixed ratio: {:.2} ({} fast, {} efficient)",
            mixed_ratio,
            (num_threads as f32 * mixed_ratio) as usize,
            num_threads - (num_threads as f32 * mixed_ratio) as usize
        );
    }
    println!("Creating pool with {} threads...", num_threads);

    let pool = create_pool_for_policy(policy, num_threads, mixed_ratio);

    let num_work_items = num_threads * 4;
    let work_size = 50000;
    let num_runs = 50;

    let mut timings = Vec::new();

    for run in 1..=num_runs {
        print!("  Run {}/{}...", run, num_runs);

        let timer = PrecisionTimer::start();

        // Execute parallel work on custom pool
        let results: Vec<u64> = pool.install(|| {
            (0..num_work_items)
                .into_par_iter()
                .map(|_| count_primes(work_size))
                .collect()
        });

        let elapsed_ms = timer.elapsed_millis_f64();
        timings.push(elapsed_ms);

        let total_primes: u64 = results.iter().sum();
        println!(" {:.2}ms (found {} primes)", elapsed_ms, total_primes);
    }

    BenchmarkStats::from_samples(&timings)
}

fn main() {
    println!("=== QoS Scheduling Test ===");
    println!("Platform: macOS (Apple Silicon)");

    // Test each policy
    let policies = vec![
        ("None (baseline)", CorePolicy::None, 0.0),
        ("FastBias", CorePolicy::FastBias, 0.0),
        ("EfficientBias", CorePolicy::EfficientBias, 0.0),
        ("Mixed (80% fast)", CorePolicy::Mixed, 0.80),
    ];

    let mut times = Vec::new();

    for (name, policy, ratio) in &policies {
        println!("Testing {}", name);
        let stats = test_single_policy(*policy, 8, *ratio);
        times.push((name, stats.median));
        println!("  Median: {:.2}ms\n", stats.median);
    }

    println!("=== Summary ===");
    let baseline = times[0].1;
    for (name, time) in &times {
        let speedup = baseline / time;
        println!("{}: {:.0}ms ({:.2}x)", name, time, speedup);
    }

    let fast_time = times[1].1;
    let efficient_time = times[2].1;
    let mixed_time = if times.len() > 3 { times[3].1 } else { 0.0 };

    println!("\n=== Key Results ===");
    println!(
        "FastBias is {:.2}x faster than EfficientBias",
        efficient_time / fast_time
    );

    if mixed_time > 0.0 {
        println!(
            "Mixed (80%) is {:.2}x faster than EfficientBias",
            efficient_time / mixed_time
        );
        println!(
            "Mixed (80%) is {:.2}x slower than FastBias",
            mixed_time / fast_time
        );

        let theoretical_mixed = 0.80 * fast_time + 0.25 * efficient_time;
        let mixed_efficiency = theoretical_mixed / mixed_time;
        println!("Mixed efficiency vs theoretical: {:.2}x", mixed_efficiency);
    }
}
