use std::time::{Duration, Instant};

use futures::stream::{self, StreamExt};

use crate::cli::Config;
use crate::error::AppError;
use crate::model::SampleOutcome;
use crate::ssh::client::{connect_authenticated, disconnect};
use crate::ssh::session::{classify_error, execute_command};
use crate::target::Target;

pub async fn run(config: &Config, targets: &[Target]) -> Result<Vec<SampleOutcome>, AppError> {
    let total = config.parallel + config.number + config.warmup;
    let identity_path = config.identity_path.clone();
    let command = config.command.clone();
    let targets = targets.to_vec();

    let mut indexed_samples = stream::iter(0..total)
        .map(|iteration| {
            let identity_path = identity_path.clone();
            let command = command.clone();
            let targets = targets.clone();

            async move {
                let worker = iteration % config.parallel;
                let target = crate::bench::select_target(&targets, worker, iteration)?;
                let started = Instant::now();

                let sample = match connect_authenticated(&target, &identity_path).await {
                    Ok(mut session) => {
                        let command_result =
                            execute_command(&session, &command, Duration::from_secs(5)).await;
                        disconnect(&mut session).await?;
                        match command_result {
                            Ok((_status, missing_exit_status, _bytes_read)) => SampleOutcome {
                                target,
                                success: true,
                                metric_value: Some(started.elapsed().as_secs_f64() * 1000.0),
                                bytes_transferred: 0,
                                missing_exit_status,
                                error_kind: None,
                                error: None,
                            },
                            Err(error) => SampleOutcome {
                                target,
                                success: false,
                                metric_value: None,
                                bytes_transferred: 0,
                                missing_exit_status: false,
                                error_kind: Some(classify_error(&error)),
                                error: Some(error.to_string()),
                            },
                        }
                    }
                    Err(error) => SampleOutcome {
                        target,
                        success: false,
                        metric_value: None,
                        bytes_transferred: 0,
                        missing_exit_status: false,
                        error_kind: Some(classify_error(&error)),
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
