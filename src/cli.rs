use std::path::PathBuf;

use clap::Parser;

use crate::target::{parse_target, Target};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BenchmarkKind {
    Auth,
    Session,
    Command,
    Throughput,
}

impl BenchmarkKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auth => "auth",
            Self::Session => "session",
            Self::Command => "command",
            Self::Throughput => "throughput",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TargetInput {
    Single(Target),
    List(PathBuf),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub parallel: usize,
    pub number: usize,
    pub warmup: usize,
    pub kind: BenchmarkKind,
    pub target_input: TargetInput,
    pub identity_path: PathBuf,
    pub command: String,
    pub throughput_command: String,
    pub size_bytes: u64,
    pub file: String,
    pub json: bool,
}

#[derive(Debug, Parser)]
#[command(name = "ssh-bench")]
pub struct Cli {
    #[arg(short = 'p', long = "parallel")]
    pub parallel: usize,

    #[arg(short = 'n', long = "number")]
    pub number: usize,

    #[arg(short = 'w', long = "warmup", default_value_t = 0)]
    pub warmup: usize,

    #[arg(short = 't', long = "type")]
    pub kind: BenchmarkKindArg,

    #[arg(short = 'c', long = "connect")]
    pub connect: Option<String>,

    #[arg(short = 'C', long = "connect-list")]
    pub connect_list: Option<PathBuf>,

    #[arg(short = 'i', long = "identity")]
    pub identity: PathBuf,

    #[arg(long = "command", default_value = "true")]
    pub command: String,

    #[arg(long = "throughput-command", default_value = "dd if={file} bs=1M count={count}")]
    pub throughput_command: String,

    #[arg(long = "size", default_value = "1GiB")]
    pub size: String,

    #[arg(long = "file", default_value = "/dev/zero")]
    pub file: String,

    #[arg(long = "json", default_value_t = false)]
    pub json: bool,
}

impl Cli {
    pub fn into_config(self) -> Result<Config, String> {
        let target_input = match (self.connect, self.connect_list) {
            (Some(target), None) => TargetInput::Single(parse_target(&target)?),
            (None, Some(path)) => TargetInput::List(path),
            (Some(_), Some(_)) => return Err("--connect and --connect-list are mutually exclusive".to_string()),
            (None, None) => return Err("either --connect or --connect-list is required".to_string()),
        };

        Ok(Config {
            parallel: self.parallel,
            number: self.number,
            warmup: self.warmup,
            kind: self.kind.into(),
            target_input,
            identity_path: self.identity,
            command: self.command,
            throughput_command: self.throughput_command,
            size_bytes: parse_size_bytes(&self.size)?,
            file: self.file,
            json: self.json,
        })
    }
}

fn parse_size_bytes(input: &str) -> Result<u64, String> {
    if let Some(value) = input.strip_suffix("GiB") {
        return value
            .parse::<u64>()
            .map(|size| size * 1024 * 1024 * 1024)
            .map_err(|_| "invalid GiB size".to_string());
    }

    if let Some(value) = input.strip_suffix("MiB") {
        return value
            .parse::<u64>()
            .map(|size| size * 1024 * 1024)
            .map_err(|_| "invalid MiB size".to_string());
    }

    input
        .parse::<u64>()
        .map_err(|_| "size must be bytes, MiB or GiB".to_string())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum BenchmarkKindArg {
    Auth,
    Session,
    Command,
    Throughput,
}

impl From<BenchmarkKindArg> for BenchmarkKind {
    fn from(value: BenchmarkKindArg) -> Self {
        match value {
            BenchmarkKindArg::Auth => Self::Auth,
            BenchmarkKindArg::Session => Self::Session,
            BenchmarkKindArg::Command => Self::Command,
            BenchmarkKindArg::Throughput => Self::Throughput,
        }
    }
}
