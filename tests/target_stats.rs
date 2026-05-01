use ssh_bench::stats::compute_latency_summary;
use ssh_bench::stats::select_measured_window;

#[test]
fn exports_stats_module() {
    let samples = vec![1.0_f64, 2.0, 3.0];
    let summary = compute_latency_summary(&samples).unwrap();
    assert_eq!(summary.min, 1.0);
}

#[test]
fn selects_middle_window_after_warmup_and_parallel() {
    let values = vec![1_u64, 2, 3, 4, 5, 6, 7];
    let window = select_measured_window(&values, 2, 2, 3);
    assert_eq!(window, vec![3, 4, 5]);
}
