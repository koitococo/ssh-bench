pub mod auth;
pub mod command;
pub mod session;
pub mod throughput;

use std::time::Instant;

use crate::cli::{BenchmarkKind, Config, TargetInput};
use crate::error::AppError;
use crate::model::BenchmarkReport;
use crate::target::{load_targets, pick_target_for_worker, Target};

pub async fn execute(config: &Config) -> Result<BenchmarkReport, AppError> {
    let targets = resolve_targets(&config.target_input)?;
    let started = Instant::now();
    let samples = match config.kind {
        BenchmarkKind::Auth => auth::run(config, &targets).await?,
        BenchmarkKind::Session => session::run(config, &targets).await?,
        BenchmarkKind::Command => command::run(config, &targets).await?,
        BenchmarkKind::Throughput => throughput::run(config, &targets).await?,
    };

    Ok(BenchmarkReport::from_samples(
        config.kind.clone(),
        &samples,
        started.elapsed().as_secs_f64() * 1000.0,
    ))
}

pub fn select_runner_kind(kind: BenchmarkKind) -> &'static str {
    kind.as_str()
}

pub fn resolve_targets(input: &TargetInput) -> Result<Vec<Target>, AppError> {
    match input {
        TargetInput::Single(target) => Ok(vec![target.clone()]),
        TargetInput::List(path) => load_targets(path),
    }
}

pub fn select_target(targets: &[Target], worker_index: usize, iteration: usize) -> Result<Target, AppError> {
    pick_target_for_worker(targets, worker_index, iteration)
        .ok_or_else(|| AppError::Config("no targets available".to_string()))
}
