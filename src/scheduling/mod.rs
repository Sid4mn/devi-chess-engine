//! Thread scheduling policies for heterogeneous architectures
//! Implements Apple QoS biasing on macOS, graceful fallback elsewhere
// QOS_CLASS_USER_INITIATED biases toward P-cores (~90% effective)
// TODO: Linux version with pthread_setaffinity would be cleaner

use clap::ValueEnum;
#[cfg(target_os = "macos")]
use libc::{pthread_set_qos_class_self_np, qos_class_t};
use rayon::{ThreadPool, ThreadPoolBuilder};
use std::error::Error;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CorePolicy {
    #[value(name = "none")]
    None,
    #[value(name = "fast")]
    FastBias,
    #[value(name = "efficient")]
    EfficientBias,
    #[value(name = "mixed")]
    Mixed,
}

pub struct HeterogeneousScheduler {
    policy: CorePolicy,
    num_threads: usize,
    mixed_ratio: f32,
}

impl HeterogeneousScheduler {
    pub fn new(policy: CorePolicy, num_threads: usize, mixed_ratio: f32) -> Self {
        HeterogeneousScheduler {
            policy: policy,
            num_threads: num_threads,
            mixed_ratio: mixed_ratio,
        }
    }

    pub fn create_thread_pool(&self) -> Result<ThreadPool, Box<dyn Error>> {
        let policy_copy = self.policy;
        let threads_copy = self.num_threads;
        let ratio_copy = self.mixed_ratio;

        ThreadPoolBuilder::new()
            .num_threads(self.num_threads)
            .thread_name(|index| format!("devi-worker-{}", index))
            .start_handler(move |index| {
                #[cfg(target_os = "macos")]
                apply_qos_for_thread(policy_copy, index, threads_copy, ratio_copy);

                if index == 0 {
                    eprintln!(
                        "Applied policy {:?} to {} threads",
                        policy_copy, threads_copy
                    );
                }
            })
            .build()
            .map_err(|e| Box::new(e) as Box<dyn Error>)
    }
}

#[cfg(target_os = "macos")]
fn apply_qos_for_thread(
    policy: CorePolicy,
    worker_index: usize,
    total_threads: usize,
    mixed_ratio: f32,
) {
    let qos_class = match policy {
        CorePolicy::None => return,
        CorePolicy::FastBias => qos_class_t::QOS_CLASS_USER_INITIATED,
        CorePolicy::EfficientBias => qos_class_t::QOS_CLASS_BACKGROUND,
        CorePolicy::Mixed => {
            let fast_workers = (total_threads as f32 * mixed_ratio) as usize;
            if worker_index < fast_workers {
                qos_class_t::QOS_CLASS_USER_INITIATED
            } else {
                qos_class_t::QOS_CLASS_BACKGROUND
            }
        }
    };

    unsafe {
        pthread_set_qos_class_self_np(qos_class, 0);
    }
}

pub fn create_pool_for_policy(
    policy: CorePolicy,
    threads: usize,
    mixed_ratio: f32,
) -> rayon::ThreadPool {
    let scheduler = HeterogeneousScheduler::new(policy, threads, mixed_ratio);

    match scheduler.create_thread_pool() {
        Ok(pool) => pool,
        Err(e) => {
            eprintln!(
                "Warning, Failed to create custom pool: {}. Using default.",
                e
            );

            ThreadPoolBuilder::new()
                .num_threads(threads)
                .build()
                .unwrap()
        }
    }
}

pub fn create_pool_for_policy_simple(policy: CorePolicy, threads: usize) -> rayon::ThreadPool {
    create_pool_for_policy(policy, threads, 0.80) // Default 80% fast cores
}

/// Create P-core biased pool (high QoS)
pub fn create_p_core_pool(num_threads: usize) -> Result<rayon::ThreadPool, rayon::ThreadPoolBuildError> {
    ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .thread_name(|i| format!("p-core-{}", i))
        .spawn_handler(|thread| {
            std::thread::spawn(move || {
                // Set high QoS on macOS
                #[cfg(target_os = "macos")]
                {
                    use std::os::raw::c_uint;
                    extern "C" {
                        fn pthread_set_qos_class_self_np(qos_class: c_uint, relative_priority: i32) -> i32;
                    }
                    const QOS_CLASS_USER_INITIATED: c_uint = 0x19;
                    unsafe { pthread_set_qos_class_self_np(QOS_CLASS_USER_INITIATED, 0); }
                }
                thread.run();
            });
            Ok(())
        })
        .build()
}

/// Create E-core biased pool (background QoS)
pub fn create_e_core_pool(num_threads: usize) -> Result<rayon::ThreadPool, rayon::ThreadPoolBuildError> {
    ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .thread_name(|i| format!("e-core-{}", i))
        .spawn_handler(|thread| {
            std::thread::spawn(move || {
                #[cfg(target_os = "macos")]
                {
                    use std::os::raw::c_uint;
                    extern "C" {
                        fn pthread_set_qos_class_self_np(qos_class: c_uint, relative_priority: i32) -> i32;
                    }
                    const QOS_CLASS_BACKGROUND: c_uint = 0x09;
                    unsafe { pthread_set_qos_class_self_np(QOS_CLASS_BACKGROUND, 0); }
                }
                thread.run();
            });
            Ok(())
        })
        .build()
}