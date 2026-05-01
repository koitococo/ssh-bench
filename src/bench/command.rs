use std::time::{Duration, Instant};

use crate::cli::Config;
use crate::error::AppError;
use crate::model::SampleOutcome;
use crate::ssh::client::{connect_authenticated, disconnect};
use crate::ssh::session::execute_command;
use crate::target::Target;

pub async fn run(config: &Config, targets: &[Target]) -> Result<Vec<SampleOutcome>, AppError> {
    let total = config.parallel + config.number + config.warmup;
    let mut samples = Vec::with_capacity(total);

    for iteration in 0..total {
        let worker = iteration % config.parallel.max(1);
        let target = crate::bench::select_target(targets, worker, iteration)?;
        let started = Instant::now();

        match connect_authenticated(&target, &config.identity_path).await {
            Ok(mut session) => {
                let command_result =
                    execute_command(&session, &config.command, Duration::from_secs(5)).await;
                disconnect(&mut session).await?;
                match command_result {
                    Ok((_status, missing_exit_status, _bytes_read)) => {
                        samples.push(SampleOutcome {
                            target,
                            success: true,
                            metric_value: Some(started.elapsed().as_secs_f64() * 1000.0),
                            bytes_transferred: 0,
                            missing_exit_status,
                            error: None,
                        })
                    }
                    Err(error) => samples.push(SampleOutcome {
                        target,
                        success: false,
                        metric_value: None,
                        bytes_transferred: 0,
                        missing_exit_status: false,
                        error: Some(error.to_string()),
                    }),
                }
            }
            Err(error) => samples.push(SampleOutcome {
                target,
                success: false,
                metric_value: None,
                bytes_transferred: 0,
                missing_exit_status: false,
                error: Some(error.to_string()),
            }),
        }
    }

    Ok(samples)
}
