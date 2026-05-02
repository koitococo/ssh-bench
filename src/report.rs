use crate::error::AppError;
use crate::model::BenchmarkReport;

const KIB: f64 = 1024.0;
const MIB: f64 = 1024.0 * 1024.0;
const GIB: f64 = 1024.0 * 1024.0 * 1024.0;

pub fn render_json_report(report: &BenchmarkReport) -> Result<String, AppError> {
    serde_json::to_string_pretty(report).map_err(|error| AppError::Config(error.to_string()))
}

pub fn render_text_report(report: &BenchmarkReport) -> String {
    let mut lines = vec![
        format!("benchmark: {}", report.benchmark),
        format!("success_count: {}", report.success_count),
        format!("failure_count: {}", report.failure_count),
        format!("wall_clock: {}", format_duration_ms(report.wall_clock_ms)),
    ];

    if let Some(summary) = &report.summary {
        if report.benchmark == "throughput" {
            lines.extend([
                format!("min_rate: {}", format_rate(summary.min)),
                format!("max_rate: {}", format_rate(summary.max)),
                format!("avg_rate: {}", format_rate(summary.avg)),
                format!("p50_rate: {}", format_rate(summary.p50)),
                format!("p99_rate: {}", format_rate(summary.p99)),
            ]);
        } else {
            lines.extend([
                format!("min: {}", format_duration_ms(summary.min)),
                format!("max: {}", format_duration_ms(summary.max)),
                format!("avg: {}", format_duration_ms(summary.avg)),
                format!("p50: {}", format_duration_ms(summary.p50)),
                format!("p99: {}", format_duration_ms(summary.p99)),
            ]);
        }
    }

    if let Some(setup_summary) = &report.setup_summary {
        lines.extend([
            format!("setup_min: {}", format_duration_ms(setup_summary.min)),
            format!("setup_max: {}", format_duration_ms(setup_summary.max)),
            format!("setup_avg: {}", format_duration_ms(setup_summary.avg)),
            format!("setup_p50: {}", format_duration_ms(setup_summary.p50)),
            format!("setup_p99: {}", format_duration_ms(setup_summary.p99)),
        ]);
    }

    if let Some(total_bytes) = report.total_bytes {
        lines.push(format!("total_bytes: {}", format_bytes(total_bytes as f64)));
    }

    if let Some(aggregate_rate) = report.aggregate_rate {
        lines.push(format!("aggregate_rate: {}", format_rate(aggregate_rate)));
    }

    if let Some(success_rate) = report.success_rate {
        lines.push(format!("success_rate: {:.3} /s", success_rate));
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
            lines.push(format!("  {}: {}", kind.as_str(), count));
        }
    }

    lines.join("\n")
}

fn format_duration_ms(duration_ms: f64) -> String {
    if duration_ms < 1000.0 {
        return format!("{duration_ms:.3} ms");
    }

    if duration_ms < 60_000.0 {
        return format!("{:.3} s", duration_ms / 1000.0);
    }

    format!("{:.3} min", duration_ms / 60_000.0)
}

fn format_rate(bytes_per_ms: f64) -> String {
    format!("{}/s", format_bytes(bytes_per_ms * 1000.0))
}

fn format_bytes(bytes: f64) -> String {
    if bytes >= GIB {
        return format!("{:.3} GiB", bytes / GIB);
    }

    if bytes >= MIB {
        return format!("{:.3} MiB", bytes / MIB);
    }

    if bytes >= KIB {
        return format!("{:.3} KiB", bytes / KIB);
    }

    format!("{bytes:.3} B")
}
