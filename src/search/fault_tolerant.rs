use crate::types::Move;
use std::panic::{catch_unwind, AssertUnwindSafe};

// Track if we've already injected panic in this recovery call
thread_local! {
    static PANIC_INJECTED: std::cell::Cell<bool> = std::cell::Cell::new(false);
}

/// Recovery wrapper catches panics and retries once
pub fn with_recovery<F>(search_fn: F, inject_panic: Option<usize>) -> (Move, i32)
where
    F: Fn() -> (Move, i32) + Send + Sync,
{
    // Reset panic flag for this recovery call
    PANIC_INJECTED.with(|flag| flag.set(false));

    // First attempt: may inject panic
    let result = catch_unwind(AssertUnwindSafe(&search_fn));

    match result {
        Ok(mv_score) => mv_score,
        Err(e) => {
            if inject_panic.is_some() {
                let msg = if let Some(s) = e.downcast_ref::<&str>() {
                    (*s).to_string()
                } else if let Some(s) = e.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "Unknown panic".to_string()
                };
                println!("  Fault detected: {}", msg);
                println!("  Retrying search...");
            }

            // Mark that we've already panicked
            PANIC_INJECTED.with(|flag| flag.set(true));

            // Retry: should NOT panic again
            search_fn()
        }
    }
}

/// Check if we should inject panic (used by parallel_search_with_fault)
pub fn should_inject_panic() -> bool {
    PANIC_INJECTED.with(|flag| !flag.get())
}
