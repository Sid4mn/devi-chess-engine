use std::time::{Duration, Instant};

/// High-precision timer for benchmarking
pub struct PrecisionTimer {
    start: Instant,
}

impl PrecisionTimer {
    pub fn start() -> Self {
        PrecisionTimer {
            start: Instant::now(),
        }
    }

    pub fn elapsed_micros(&self) -> u128 {
        self.start.elapsed().as_micros()
    }

    pub fn elapsed_millis_f64(&self) -> f64 {
        self.start.elapsed().as_micros() as f64 / 1000.0
    }
}

/// Measure execution time of a closure
pub fn time_execution<F, R>(f: F) -> (R, Duration)
where
    F: FnOnce() -> R,
{
    let start = Instant::now();
    let result = f();
    let duration = start.elapsed();
    (result, duration)
}

/// Measure execution time in milliseconds with microsecond precision
pub fn time_execution_millis<F, R>(f: F) -> (R, f64)
where
    F: FnOnce() -> R,
{
    let start = Instant::now();
    let result = f();
    let duration_ms = start.elapsed().as_micros() as f64 / 1000.0;
    (result, duration_ms)
}
