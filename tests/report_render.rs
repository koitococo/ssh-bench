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
        setup_time_ms: None,
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
        setup_time_ms: None,
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
        setup_time_ms: None,
        bytes_transferred: 0,
        missing_exit_status: false,
        error_kind: Some(ErrorKind::CommandTimeout),
        error: Some("timeout".to_string()),
    };

    let report = BenchmarkReport::from_samples(BenchmarkKind::Command, &[failed], 1000.0, 0, 1, 1);
    let rendered = render_text_report(&report);

    assert!(rendered.contains("error_counts:"));
    assert!(rendered.contains("CommandTimeout: 1"));
}

#[test]
fn renders_aggregate_throughput_field() {
    let sample = SampleOutcome {
        target: Target::new("u", "h", 22),
        success: true,
        metric_value: Some(4.0),
        setup_time_ms: Some(12.0),
        bytes_transferred: 4000,
        missing_exit_status: false,
        error_kind: None,
        error: None,
    };

    let report = BenchmarkReport::from_samples(BenchmarkKind::Throughput, &[sample], 1000.0, 0, 1, 0);
    let rendered = render_text_report(&report);

    assert!(rendered.contains("aggregate_rate_bytes_per_ms:"));
    assert!(rendered.contains("setup_avg_ms:"));
    assert!(rendered.contains("p50_bytes_per_ms:"));
}

#[test]
fn trims_latency_window_before_filtering_successes() {
    let samples = vec![
        SampleOutcome {
            target: Target::new("u", "h", 22),
            success: false,
            metric_value: None,
            setup_time_ms: None,
            bytes_transferred: 0,
            missing_exit_status: false,
            error_kind: Some(ErrorKind::CommandTimeout),
            error: Some("warmup fail".to_string()),
        },
        SampleOutcome {
            target: Target::new("u", "h", 22),
            success: true,
            metric_value: Some(10.0),
            setup_time_ms: None,
            bytes_transferred: 0,
            missing_exit_status: false,
            error_kind: None,
            error: None,
        },
        SampleOutcome {
            target: Target::new("u", "h", 22),
            success: true,
            metric_value: Some(30.0),
            setup_time_ms: None,
            bytes_transferred: 0,
            missing_exit_status: false,
            error_kind: None,
            error: None,
        },
        SampleOutcome {
            target: Target::new("u", "h", 22),
            success: false,
            metric_value: None,
            setup_time_ms: None,
            bytes_transferred: 0,
            missing_exit_status: false,
            error_kind: Some(ErrorKind::CommandTimeout),
            error: Some("tail fail".to_string()),
        },
    ];

    let report = BenchmarkReport::from_samples(BenchmarkKind::Command, &samples, 1000.0, 1, 1, 2);
    let summary = report.summary.expect("summary expected");

    assert_eq!(summary.min, 10.0);
    assert_eq!(summary.max, 30.0);
}

#[test]
fn reports_missing_exit_status_count() {
    let failed = SampleOutcome {
        target: Target::new("u", "h", 22),
        success: false,
        metric_value: None,
        setup_time_ms: None,
        bytes_transferred: 0,
        missing_exit_status: true,
        error_kind: Some(ErrorKind::CommandTimeout),
        error: Some("timeout after eof fallback".to_string()),
    };

    let report = BenchmarkReport::from_samples(BenchmarkKind::Command, &[failed], 1000.0, 0, 1, 1);
    let rendered = render_text_report(&report);

    assert!(rendered.contains("missing_exit_status: 1"));
}

#[test]
fn reports_missing_exit_status_count_for_throughput() {
    let failed = SampleOutcome {
        target: Target::new("u", "h", 22),
        success: false,
        metric_value: None,
        setup_time_ms: None,
        bytes_transferred: 0,
        missing_exit_status: true,
        error_kind: Some(ErrorKind::ReadTimeout),
        error: Some("stream closed without exit status".to_string()),
    };

    let report = BenchmarkReport::from_samples(BenchmarkKind::Throughput, &[failed], 1000.0, 0, 1, 0);
    let rendered = render_text_report(&report);

    assert!(rendered.contains("missing_exit_status: 1"));
}
