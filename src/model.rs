use std::collections::BTreeMap;

use serde::Serialize;

use crate::cli::BenchmarkKind;
use crate::error::ErrorKind;
use crate::stats::LatencySummary;
use crate::stats::compute_latency_summary;
use crate::stats::select_measured_window;
use crate::target::Target;

#[derive(Debug, Clone)]
pub struct SampleOutcome {
    pub target: Target,
    pub success: bool,
    pub metric_value: Option<f64>,
    pub setup_time_ms: Option<f64>,
    pub bytes_transferred: u64,
    pub missing_exit_status: bool,
    pub error_kind: Option<ErrorKind>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkReport {
    pub benchmark: &'static str,
    pub success_count: usize,
    pub failure_count: usize,
    pub wall_clock_ms: f64,
    pub aggregate_rate: Option<f64>,
    pub success_rate: Option<f64>,
    pub total_bytes: Option<u64>,
    pub summary: Option<LatencySummary>,
    pub setup_summary: Option<LatencySummary>,
    pub missing_exit_status: usize,
    pub error_counts: BTreeMap<ErrorKind, usize>,
    pub errors: Vec<String>,
}

impl BenchmarkReport {
    pub fn from_samples(
        kind: BenchmarkKind,
        samples: &[SampleOutcome],
        wall_clock_ms: f64,
        warmup: usize,
        parallel: usize,
        number: usize,
    ) -> Self {
        let measured_samples = if matches!(kind, BenchmarkKind::Throughput) {
            samples.to_vec()
        } else {
            select_measured_window(samples, warmup, parallel, number)
        };

        let benchmark = kind.as_str();
        let success_count = measured_samples
            .iter()
            .filter(|sample| sample.success)
            .count();
        let failure_count = measured_samples.len().saturating_sub(success_count);
        let missing_exit_status = measured_samples
            .iter()
            .filter(|sample| sample.missing_exit_status)
            .count();
        let error_counts = measured_samples
            .iter()
            .filter_map(|sample| sample.error_kind.clone())
            .fold(BTreeMap::new(), |mut counts, kind| {
                *counts.entry(kind).or_insert(0) += 1;
                counts
            });
        let errors = measured_samples
            .iter()
            .filter_map(|sample| sample.error.clone())
            .collect::<Vec<_>>();
        let measured_metrics = if matches!(kind, BenchmarkKind::Throughput) {
            measured_samples
                .iter()
                .filter(|sample| sample.success)
                .filter_map(|sample| sample.metric_value)
                .collect::<Vec<_>>()
        } else {
            measured_samples
                .iter()
                .filter(|sample| sample.success)
                .filter_map(|sample| sample.metric_value)
                .collect::<Vec<_>>()
        };
        let summary = compute_latency_summary(&measured_metrics);
        let setup_metrics = if matches!(kind, BenchmarkKind::Throughput) {
            measured_samples
                .iter()
                .filter(|sample| sample.success)
                .filter_map(|sample| sample.setup_time_ms)
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        let setup_summary = compute_latency_summary(&setup_metrics);
        let total_bytes = measured_samples
            .iter()
            .filter(|sample| sample.success)
            .map(|sample| sample.bytes_transferred)
            .sum::<u64>();
        let aggregate_rate = if matches!(kind, BenchmarkKind::Throughput) && wall_clock_ms > 0.0 {
            Some(total_bytes as f64 / wall_clock_ms)
        } else {
            None
        };
        let success_rate = if !matches!(kind, BenchmarkKind::Throughput) && wall_clock_ms > 0.0 {
            Some(success_count as f64 / (wall_clock_ms / 1000.0))
        } else {
            None
        };

        Self {
            benchmark,
            success_count,
            failure_count,
            wall_clock_ms,
            aggregate_rate,
            success_rate,
            total_bytes: matches!(kind, BenchmarkKind::Throughput).then_some(total_bytes),
            summary,
            setup_summary,
            missing_exit_status,
            error_counts,
            errors,
        }
    }
}
