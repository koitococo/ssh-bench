use std::time::{Duration, Instant};

use futures::future::join_all;

use crate::cli::Config;
use crate::error::AppError;
use crate::error::ErrorKind;
use crate::model::SampleOutcome;
use crate::ssh::client::{connect_authenticated, disconnect};
use crate::ssh::session::{read_throughput, render_throughput_command};
use crate::target::Target;

pub async fn run(config: &Config, targets: &[Target]) -> Result<Vec<SampleOutcome>, AppError> {
    let identity_path = config.identity_path.clone();
    let throughput_command = config.throughput_command.clone();
    let file = config.file.clone();
    let size_bytes = config.size_bytes;
    let targets = targets.to_vec();

    let tasks = (0..config.parallel.max(1)).map(|worker| {
        let identity_path = identity_path.clone();
        let throughput_command = throughput_command.clone();
        let file = file.clone();
        let targets = targets.clone();

        async move {
            let target = crate::bench::select_target(&targets, worker, 0)?;
            let command = render_throughput_command(&throughput_command, &file, size_bytes)
                .map_err(AppError::Config)?;

            let sample = match connect_authenticated(&target, &identity_path).await {
                Ok(mut session) => {
                    let read_started = Instant::now();
                    let throughput_result =
                        read_throughput(&session, &command, size_bytes, Duration::from_secs(5))
                            .await;
                    disconnect(&mut session).await?;
                    match throughput_result {
                        Ok((bytes_read, _status, missing_exit_status)) => {
                            let elapsed_ms = read_started.elapsed().as_secs_f64() * 1000.0;
                            let rate = if elapsed_ms > 0.0 {
                                bytes_read as f64 / elapsed_ms
                            } else {
                                0.0
                            };

                            SampleOutcome {
                                target,
                                success: true,
                                metric_value: Some(rate),
                                bytes_transferred: bytes_read,
                                missing_exit_status,
                                error_kind: None,
                                error: None,
                            }
                        }
                        Err(error) => SampleOutcome {
                            target,
                            success: false,
                            metric_value: None,
                            bytes_transferred: 0,
                            missing_exit_status: false,
                            error_kind: Some(error.kind()),
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
                    error_kind: Some(ErrorKind::Ssh),
                    error: Some(error.to_string()),
                },
            };

            Ok::<SampleOutcome, AppError>(sample)
        }
    });

    join_all(tasks).await.into_iter().collect()
}
