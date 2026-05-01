use ssh_bench::cli::BenchmarkKind;
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
        error: None,
    };

    let report = BenchmarkReport::from_samples(BenchmarkKind::Auth, &[sample], 120.0);
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
        error: None,
    };

    let report = BenchmarkReport::from_samples(BenchmarkKind::Auth, &[sample], 120.0);
    let rendered = render_text_report(&report);

    assert!(rendered.contains("benchmark: auth"));
    assert!(rendered.contains("success_count: 1"));
}
