use crate::types::Move;
use std::panic::{catch_unwind, AssertUnwindSafe};

// outer safety net - per-move recovery handles faults now, this just catches stragglers.
pub fn with_recovery<F>(search_fn: F, inject_panic: Option<usize>) -> (Move, i32)
where
    F: Fn() -> (Move, i32) + Send + Sync,
{
    match catch_unwind(AssertUnwindSafe(&search_fn)) {
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

            search_fn()
        }
    }
}
