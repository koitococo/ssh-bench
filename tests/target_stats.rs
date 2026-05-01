use ssh_bench::stats::compute_latency_summary;

#[test]
fn exports_stats_module() {
    let samples = vec![1.0_f64, 2.0, 3.0];
    let summary = compute_latency_summary(&samples).unwrap();
    assert_eq!(summary.min, 1.0);
}
