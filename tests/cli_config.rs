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
