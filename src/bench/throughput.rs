use std::time::{Duration, Instant};

use crate::cli::Config;
use crate::error::AppError;
use crate::model::SampleOutcome;
use crate::ssh::client::{connect_authenticated, disconnect};
use crate::ssh::session::{read_throughput, render_throughput_command};
use crate::target::Target;

pub async fn run(config: &Config, targets: &[Target]) -> Result<Vec<SampleOutcome>, AppError> {
    let mut samples = Vec::with_capacity(config.parallel.max(1));

    for worker in 0..config.parallel.max(1) {
        let target = crate::bench::select_target(targets, worker, 0)?;
        let started = Instant::now();
        let command =
            render_throughput_command(&config.throughput_command, &config.file, config.size_bytes)
                .map_err(AppError::Config)?;

        match connect_authenticated(&target, &config.identity_path).await {
            Ok(mut session) => {
                let throughput_result = read_throughput(
                    &session,
                    &command,
                    config.size_bytes,
                    Duration::from_secs(5),
                )
                .await;
                disconnect(&mut session).await?;
                match throughput_result {
                    Ok((bytes_read, _status, missing_exit_status)) => {
                        let elapsed_ms = started.elapsed().as_secs_f64() * 1000.0;
                        let rate = if elapsed_ms > 0.0 {
                            bytes_read as f64 / elapsed_ms
                        } else {
                            0.0
                        };

                        samples.push(SampleOutcome {
                            target,
                            success: true,
                            metric_value: Some(rate),
                            bytes_transferred: bytes_read,
                            missing_exit_status,
                            error: None,
                        });
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
