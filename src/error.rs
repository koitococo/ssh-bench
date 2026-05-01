use std::io;

use thiserror::Error;

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
