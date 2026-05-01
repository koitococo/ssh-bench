use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LatencySummary {
    pub min: f64,
    pub max: f64,
    pub avg: f64,
    pub p50: f64,
    pub p99: f64,
}

pub fn compute_latency_summary(samples: &[f64]) -> Option<LatencySummary> {
    if samples.is_empty() {
        return None;
    }

    let mut sorted = samples.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let min = *sorted.first()?;
    let max = *sorted.last()?;
    let avg = sorted.iter().sum::<f64>() / sorted.len() as f64;
    let p50 = percentile(&sorted, 0.50);
    let p99 = percentile(&sorted, 0.99);

    Some(LatencySummary {
        min,
        max,
        avg,
        p50,
        p99,
    })
}

pub fn select_measured_window<T: Clone>(
    values: &[T],
    warmup: usize,
    parallel: usize,
    number: usize,
) -> Vec<T> {
    let remaining = values.len().saturating_sub(warmup);
    let middle_len = remaining.saturating_sub(parallel).min(number);

    values
        .iter()
        .skip(warmup)
        .take(middle_len)
        .cloned()
        .collect()
}

fn percentile(sorted: &[f64], quantile: f64) -> f64 {
    let index = ((sorted.len() as f64 * quantile).ceil() as usize).saturating_sub(1);
    sorted[index]
}
