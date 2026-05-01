use std::io;

use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    Config,
    Target,
    KeyRead,
    KeyFormat,
    TcpConnect,
    SshHandshake,
    Authentication,
    SessionOpen,
    Exec,
    CommandTimeout,
    ReadTimeout,
    Io,
    Protocol,
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("配置错误: {0}")]
    Config(String),
    #[error("目标解析错误: {0}")]
    Target(String),
    #[error("IO 错误: {0}")]
    Io(#[from] io::Error),
    #[error("SSH 错误: {0}")]
    Ssh(#[from] russh::Error),
    #[error("私钥加载错误: {0}")]
    Key(String),
    #[error("超时: {0}")]
    Timeout(&'static str),
}

impl AppError {
    pub fn kind(&self) -> ErrorKind {
        match self {
            Self::Config(_) => ErrorKind::Config,
            Self::Target(_) => ErrorKind::Target,
            Self::Io(_) => ErrorKind::Io,
            Self::Ssh(_) => ErrorKind::Protocol,
            Self::Key(_) => ErrorKind::KeyFormat,
            Self::Timeout(_) => ErrorKind::CommandTimeout,
        }
    }
}
