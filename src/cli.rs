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
        })
    }
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
