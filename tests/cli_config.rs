use clap::Parser;
use ssh_bench::cli::{BenchmarkKind, Cli, TargetInput};
use ssh_bench::ssh::session::render_throughput_command;

#[test]
fn parses_identity_and_single_target_config() {
    let cli = Cli::parse_from([
        "ssh-bench",
        "--parallel",
        "4",
        "--number",
        "10",
        "--warmup",
        "2",
        "--type",
        "auth",
        "--connect",
        "alice@example.com:22",
        "--identity",
        "/tmp/id_ed25519",
    ]);

    let config = cli.into_config().unwrap();
    assert_eq!(config.parallel, 4);
    assert_eq!(config.number, 10);
    assert_eq!(config.warmup, 2);
    assert_eq!(config.kind, BenchmarkKind::Auth);
    assert_eq!(config.identity_path.to_string_lossy(), "/tmp/id_ed25519");

    match config.target_input {
        TargetInput::Single(target) => assert_eq!(target.host, "example.com"),
        TargetInput::List(_) => panic!("expected single target"),
    }
}

#[test]
fn renders_default_throughput_command() {
    let command = render_throughput_command(
        "dd if={file} bs=1M count={count}",
        "/dev/zero",
        1024 * 1024 * 1024,
    )
    .unwrap();

    assert_eq!(command, "dd if=/dev/zero bs=1M count=1024");
}

#[test]
fn rejects_both_connect_and_connect_list() {
    let cli = Cli::parse_from([
        "ssh-bench",
        "--parallel",
        "2",
        "--number",
        "10",
        "--type",
        "auth",
        "--connect",
        "alice@example.com:22",
        "--connect-list",
        "/tmp/targets.txt",
        "--identity",
        "/tmp/id_ed25519",
    ]);

    let error = cli.into_config().unwrap_err();
    assert!(error.contains("mutually exclusive"));
}

#[test]
fn rejects_zero_parallel() {
    let cli = Cli::parse_from([
        "ssh-bench",
        "--parallel",
        "0",
        "--number",
        "10",
        "--type",
        "auth",
        "--connect",
        "alice@example.com:22",
        "--identity",
        "/tmp/id_ed25519",
    ]);

    let error = cli.into_config().unwrap_err();
    assert!(error.contains("parallel must be greater than 0"));
}

#[test]
fn throughput_allows_zero_number_and_parses_size() {
    let cli = Cli::parse_from([
        "ssh-bench",
        "--parallel",
        "4",
        "--number",
        "0",
        "--type",
        "throughput",
        "--connect",
        "alice@example.com:22",
        "--identity",
        "/tmp/id_ed25519",
        "--size",
        "2GiB",
    ]);

    let config = cli.into_config().unwrap();
    assert_eq!(config.kind, BenchmarkKind::Throughput);
    assert_eq!(config.number, 0);
    assert_eq!(config.size_bytes, 2 * 1024 * 1024 * 1024);
}

#[test]
fn throughput_normalizes_unused_number_and_warmup() {
    let cli = Cli::parse_from([
        "ssh-bench",
        "--parallel",
        "4",
        "--number",
        "9",
        "--warmup",
        "3",
        "--type",
        "throughput",
        "--connect",
        "alice@example.com:22",
        "--identity",
        "/tmp/id_ed25519",
    ]);

    let config = cli.into_config().unwrap();

    assert_eq!(config.kind, BenchmarkKind::Throughput);
    assert_eq!(config.number, 0);
    assert_eq!(config.warmup, 0);
}

#[test]
fn latency_modes_require_positive_number() {
    let cli = Cli::parse_from([
        "ssh-bench",
        "--parallel",
        "2",
        "--number",
        "0",
        "--type",
        "auth",
        "--connect",
        "alice@example.com:22",
        "--identity",
        "/tmp/id_ed25519",
    ]);

    let error = cli.into_config().unwrap_err();
    assert!(error.contains("number must be greater than 0"));
}

#[test]
fn rounds_up_non_integral_mib_count() {
    let command = render_throughput_command(
        "dd if={file} bs=1M count={count}",
        "/tmp/data.bin",
        1024 * 1024 + 1,
    )
    .unwrap();

    assert_eq!(command, "dd if=/tmp/data.bin bs=1M count=2");
}
