use ssh_bench::stats::compute_latency_summary;
use ssh_bench::stats::select_measured_window;
use ssh_bench::target::Target;
use ssh_bench::target::load_targets;
use ssh_bench::target::parse_target;
use ssh_bench::target::pick_target_for_worker;

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

#[test]
fn parses_user_host_port_target() {
    let target = parse_target("alice@example.com:2222").unwrap();
    assert_eq!(target.user, "alice");
    assert_eq!(target.host, "example.com");
    assert_eq!(target.port, 2222);
}

#[test]
fn rotates_targets_by_worker_and_iteration() {
    let targets = vec![
        Target::new("u1", "h1", 22),
        Target::new("u2", "h2", 22),
        Target::new("u3", "h3", 22),
    ];

    let picked = pick_target_for_worker(&targets, 1, 2).unwrap();
    assert_eq!(picked.host, "h1");
}

#[test]
fn loads_targets_from_file_with_trimmed_lines() {
    let path = std::env::temp_dir().join(format!("ssh-bench-targets-{}.txt", std::process::id()));
    std::fs::write(&path, " alice@example.com:22 \n\n bob@example.com:2222 \n").unwrap();

    let targets = load_targets(&path).unwrap();

    std::fs::remove_file(&path).unwrap();

    assert_eq!(targets.len(), 2);
    assert_eq!(targets[0].user, "alice");
    assert_eq!(targets[1].port, 2222);
}
