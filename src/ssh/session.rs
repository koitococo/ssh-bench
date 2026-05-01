use std::time::Duration;

use russh::ChannelMsg;
use russh::client;
use tokio::time::timeout;

use crate::error::AppError;

const MIB: u64 = 1024 * 1024;

pub fn render_throughput_command(
    template: &str,
    file: &str,
    size_bytes: u64,
) -> Result<String, String> {
    if !template.contains("{file}") || !template.contains("{count}") {
        return Err("template must contain {file} and {count}".to_string());
    }

    let count = size_bytes.div_ceil(MIB);
    Ok(template
        .replace("{file}", file)
        .replace("{count}", &count.to_string()))
}

pub async fn open_session(
    session: &client::Handle<crate::ssh::client::AcceptAllClient>,
) -> Result<russh::Channel<client::Msg>, AppError> {
    session.channel_open_session().await.map_err(AppError::from)
}

pub async fn execute_command(
    session: &client::Handle<crate::ssh::client::AcceptAllClient>,
    command: &str,
    wait_timeout: Duration,
) -> Result<(Option<u32>, bool, u64), AppError> {
    let mut channel = open_session(session).await?;
    channel.exec(true, command).await?;

    let mut exit_status = None;
    let mut missing_exit_status = false;
    let mut bytes_read = 0_u64;

    loop {
        let message = timeout(wait_timeout, channel.wait())
            .await
            .map_err(|_| AppError::Timeout("waiting for command event timed out"))?;

        let Some(message) = message else {
            break;
        };

        match message {
            ChannelMsg::Data { ref data } => {
                bytes_read += data.len() as u64;
            }
            ChannelMsg::ExitStatus {
                exit_status: status,
            } => {
                exit_status = Some(status);
            }
            ChannelMsg::Eof | ChannelMsg::Close => {
                if exit_status.is_none() {
                    missing_exit_status = true;
                }
                break;
            }
            _ => {}
        }
    }

    channel.close().await?;
    Ok((exit_status, missing_exit_status, bytes_read))
}

pub async fn read_throughput(
    session: &client::Handle<crate::ssh::client::AcceptAllClient>,
    command: &str,
    size_limit: u64,
    wait_timeout: Duration,
) -> Result<(u64, Option<u32>, bool), AppError> {
    let mut channel = open_session(session).await?;
    channel.exec(true, command).await?;

    let mut total = 0_u64;
    let mut exit_status = None;

    loop {
        if total >= size_limit {
            break;
        }

        let message = timeout(wait_timeout, channel.wait())
            .await
            .map_err(|_| AppError::Timeout("reading throughput stream timed out"))?;

        let Some(message) = message else {
            break;
        };

        match message {
            ChannelMsg::Data { ref data } => {
                let remaining = size_limit.saturating_sub(total) as usize;
                total += data.len().min(remaining) as u64;
            }
            ChannelMsg::ExitStatus {
                exit_status: status,
            } => {
                exit_status = Some(status);
            }
            ChannelMsg::Eof | ChannelMsg::Close => {
                break;
            }
            _ => {}
        }
    }

    let missing_exit_status = exit_status.is_none();
    channel.close().await?;
    Ok((total.min(size_limit), exit_status, missing_exit_status))
}
