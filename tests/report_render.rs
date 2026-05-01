use ssh_bench::cli::BenchmarkKind;
use ssh_bench::error::ErrorKind;
use ssh_bench::model::{BenchmarkReport, SampleOutcome};
use ssh_bench::report::{render_json_report, render_text_report};
use ssh_bench::target::Target;

#[test]
fn renders_json_report() {
    let sample = SampleOutcome {
        target: Target::new("u", "h", 22),
        success: true,
        metric_value: Some(12.0),
        bytes_transferred: 0,
        missing_exit_status: false,
        error_kind: None,
        error: None,
    };

    let report = BenchmarkReport::from_samples(BenchmarkKind::Auth, &[sample], 120.0, 0, 1, 1);
    let rendered = render_json_report(&report).unwrap();

    assert!(rendered.contains("\"success_count\": 1"));
}

#[test]
fn renders_text_report() {
    let sample = SampleOutcome {
        target: Target::new("u", "h", 22),
        success: true,
        metric_value: Some(12.0),
        bytes_transferred: 0,
        missing_exit_status: false,
        error_kind: None,
        error: None,
    };

    let report = BenchmarkReport::from_samples(BenchmarkKind::Auth, &[sample], 120.0, 0, 1, 1);
    let rendered = render_text_report(&report);

    assert!(rendered.contains("benchmark: auth"));
    assert!(rendered.contains("success_count: 1"));
    assert!(rendered.contains("successes_per_second:"));
}

#[test]
fn renders_error_counts() {
    let failed = SampleOutcome {
        target: Target::new("u", "h", 22),
        success: false,
        metric_value: None,
        bytes_transferred: 0,
        missing_exit_status: false,
        error_kind: Some(ErrorKind::Timeout),
        error: Some("timeout".to_string()),
    };

    let report = BenchmarkReport::from_samples(BenchmarkKind::Command, &[failed], 1000.0, 0, 1, 1);
    let rendered = render_text_report(&report);

    assert!(rendered.contains("error_counts:"));
    assert!(rendered.contains("Timeout: 1"));
}
