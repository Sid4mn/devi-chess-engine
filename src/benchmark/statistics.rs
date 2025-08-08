/// Statistical analysis for benchmark results
#[derive(Debug, Clone)]
pub struct BenchmarkStats {
    pub samples: Vec<f64>,
    pub mean: f64,
    pub median: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub percentile_95: f64,
}

impl BenchmarkStats {
    pub fn from_samples(samples: &[f64]) -> Self {
        if samples.is_empty() {
            return BenchmarkStats {
                samples: vec![],
                mean: 0.0,
                median: 0.0,
                std_dev: 0.0,
                min: 0.0,
                max: 0.0,
                percentile_95: 0.0,
            };
        }
        
        let mut sorted = samples.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let mean = samples.iter().sum::<f64>() / samples.len() as f64;
        
        let median = if sorted.len() % 2 == 0 {
            (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) / 2.0
        } else {
            sorted[sorted.len() / 2]
        };
        
        let variance = samples.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / samples.len() as f64;
        let std_dev = variance.sqrt();
        
        let min = sorted[0];
        let max = sorted[sorted.len() - 1];
        
        let p95_index = ((sorted.len() as f64 * 0.95) as usize).min(sorted.len() - 1);
        let percentile_95 = sorted[p95_index];
        
        BenchmarkStats {
            samples: samples.to_vec(),
            mean,
            median,
            std_dev,
            min,
            max,
            percentile_95,
        }
    }
    
    pub fn searches_per_second(&self) -> f64 {
        if self.median > 0.0 {
            1000.0 / self.median
        } else {
            0.0
        }
    }
}