use std::time::Instant;

use futures::stream::{self, StreamExt};

use crate::cli::Config;
use crate::error::AppError;
use crate::model::SampleOutcome;
use crate::ssh::client::{connect_authenticated, disconnect};
use crate::target::Target;

pub async fn run(config: &Config, targets: &[Target]) -> Result<Vec<SampleOutcome>, AppError> {
    let total = config.parallel + config.number + config.warmup;
    let identity_path = config.identity_path.clone();
    let targets = targets.to_vec();

    let mut indexed_samples = stream::iter(0..total)
        .map(|iteration| {
            let identity_path = identity_path.clone();
            let targets = targets.clone();

            async move {
                let worker = iteration % config.parallel;
                let target = crate::bench::select_target(&targets, worker, iteration)?;
                let started = Instant::now();

                let sample = match connect_authenticated(&target, &identity_path).await {
                    Ok(mut session) => {
                        disconnect(&mut session).await?;
                        SampleOutcome {
                            target,
                            success: true,
                            metric_value: Some(started.elapsed().as_secs_f64() * 1000.0),
                            bytes_transferred: 0,
                            missing_exit_status: false,
                            error: None,
                        }
                    }
                    Err(error) => SampleOutcome {
                        target,
                        success: false,
                        metric_value: None,
                        bytes_transferred: 0,
                        missing_exit_status: false,
                        error: Some(error.to_string()),
                    },
                };

                Ok::<(usize, SampleOutcome), AppError>((iteration, sample))
            }
        })
        .buffer_unordered(config.parallel)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<Vec<_>, AppError>>()?;

    indexed_samples.sort_by_key(|(iteration, _)| *iteration);

    Ok(indexed_samples
        .into_iter()
        .map(|(_, sample)| sample)
        .collect())
}
