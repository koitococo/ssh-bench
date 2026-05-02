use std::time::Duration;

use russh::ChannelMsg;
use russh::client;
use tokio::time::timeout;

use crate::error::AppError;
use crate::error::ErrorKind;

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
) -> Result<(Option<u32>, bool, u64), (AppError, bool)> {
    let mut channel = open_session(session)
        .await
        .map_err(|error| (AppError::Config(format!("session_open: {}", error)), false))?;
    channel
        .exec(true, command)
        .await
        .map_err(|error| (AppError::Config(format!("exec: {}", error)), false))?;

    let mut exit_status = None;
    let mut missing_exit_status = false;
    let mut bytes_read = 0_u64;

    loop {
        let message = timeout(wait_timeout, channel.wait()).await.map_err(|_| {
            (
                AppError::Config(format!(
                    "command_timeout: {}",
                    "waiting for command event timed out"
                )),
                exit_status.is_none(),
            )
        })?;

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

    channel.close().await.map_err(|error| {
        (
            AppError::Config(format!("exec: {}", error)),
            missing_exit_status,
        )
    })?;
    Ok((exit_status, missing_exit_status, bytes_read))
}

pub async fn read_throughput(
    session: &client::Handle<crate::ssh::client::AcceptAllClient>,
    command: &str,
    size_limit: u64,
    wait_timeout: Duration,
) -> Result<(u64, Option<u32>, bool, f64, f64), (AppError, bool)> {
    let setup_started = std::time::Instant::now();
    let mut channel = open_session(session)
        .await
        .map_err(|error| (AppError::Config(format!("session_open: {}", error)), false))?;
    channel
        .exec(true, command)
        .await
        .map_err(|error| (AppError::Config(format!("exec: {}", error)), false))?;
    let setup_elapsed_ms = setup_started.elapsed().as_secs_f64() * 1000.0;
    let read_started = std::time::Instant::now();

    let mut total = 0_u64;
    let mut exit_status = None;
    let mut missing_exit_status = false;

    loop {
        if total >= size_limit {
            break;
        }

        let message = timeout(wait_timeout, channel.wait()).await.map_err(|_| {
            (
                AppError::Config(format!(
                    "read_timeout: {}",
                    "reading throughput stream timed out"
                )),
                exit_status.is_none(),
            )
        })?;

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
                if exit_status.is_none() {
                    missing_exit_status = true;
                }
                break;
            }
            _ => {}
        }
    }

    if exit_status.is_none() {
        missing_exit_status = true;
    }
    let read_elapsed_ms = read_started.elapsed().as_secs_f64() * 1000.0;
    channel.close().await.map_err(|error| {
        (
            AppError::Config(format!("exec: {}", error)),
            missing_exit_status,
        )
    })?;
    Ok((
        total.min(size_limit),
        exit_status,
        missing_exit_status,
        setup_elapsed_ms,
        read_elapsed_ms,
    ))
}

pub fn classify_error(error: &AppError) -> ErrorKind {
    match error {
        AppError::Config(message) if message.starts_with("key_read:") => ErrorKind::KeyRead,
        AppError::Config(message) if message.starts_with("tcp_connect:") => ErrorKind::TcpConnect,
        AppError::Config(message) if message.starts_with("ssh_handshake:") => {
            ErrorKind::SshHandshake
        }
        AppError::Config(message) if message.starts_with("key_format:") => ErrorKind::KeyFormat,
        AppError::Config(message) if message.starts_with("authentication:") => {
            ErrorKind::Authentication
        }
        AppError::Config(message) if message.starts_with("session_open:") => ErrorKind::SessionOpen,
        AppError::Config(message) if message.starts_with("exec:") => ErrorKind::Exec,
        AppError::Config(message) if message.starts_with("command_timeout:") => {
            ErrorKind::CommandTimeout
        }
        AppError::Config(message) if message.starts_with("read_timeout:") => ErrorKind::ReadTimeout,
        _ => error.kind(),
    }
}
