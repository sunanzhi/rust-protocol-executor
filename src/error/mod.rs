// src/error.rs
use thiserror::Error;

/// 项目错误类型
#[derive(Error, Debug)]
pub enum Error {
    /// 配置错误
    #[error("Configuration error: {0}")]
    Config(String),

    /// IO错误
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// 中间件错误
    #[error("Middleware error: {0}")]
    Middleware(String),

    /// 协议错误
    #[error("Protocol error: {0}")]
    Protocol(String),

    /// 执行器错误
    #[error("Executor error: {0}")]
    Executor(String),

    /// 序列化/反序列化错误
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// TOML解析错误
    #[error("TOML parsing error: {0}")]
    Toml(#[from] toml::de::Error),

    /// YAML解析错误
    #[error("YAML parsing error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    /// 网络错误
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    /// 其他错误
    #[error("Other error: {0}")]
    Other(String),
}

/// 项目结果类型
pub type Result<T> = std::result::Result<T, Error>;