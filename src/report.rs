use crate::error::AppError;
use crate::model::BenchmarkReport;

pub fn render_json_report(report: &BenchmarkReport) -> Result<String, AppError> {
    serde_json::to_string_pretty(report).map_err(|error| AppError::Config(error.to_string()))
}

pub fn render_text_report(report: &BenchmarkReport) -> String {
    let mut lines = vec![
        format!("benchmark: {}", report.benchmark),
        format!("success_count: {}", report.success_count),
        format!("failure_count: {}", report.failure_count),
        format!("wall_clock_ms: {:.3}", report.wall_clock_ms),
    ];

    if let Some(summary) = &report.summary {
        lines.push(format!("min_ms: {:.3}", summary.min));
        lines.push(format!("max_ms: {:.3}", summary.max));
        lines.push(format!("avg_ms: {:.3}", summary.avg));
        lines.push(format!("p50_ms: {:.3}", summary.p50));
        lines.push(format!("p99_ms: {:.3}", summary.p99));
    }

    if let Some(total_bytes) = report.total_bytes {
        lines.push(format!("total_bytes: {}", total_bytes));
    }

    if let Some(aggregate_rate) = report.aggregate_rate {
        lines.push(format!(
            "aggregate_rate_bytes_per_ms: {:.3}",
            aggregate_rate
        ));
    }

    if let Some(success_rate) = report.success_rate {
        lines.push(format!("successes_per_second: {:.3}", success_rate));
    }

    if report.missing_exit_status > 0 {
        lines.push(format!(
            "missing_exit_status: {}",
            report.missing_exit_status
        ));
    }

    if !report.error_counts.is_empty() {
        lines.push("error_counts:".to_string());
        for (kind, count) in &report.error_counts {
            lines.push(format!("  {:?}: {}", kind, count));
        }
    }

    lines.join("\n")
}
