//! 错误类型定义

use thiserror::Error;

/// spring-lsp 错误类型
#[derive(Debug, Error)]
pub enum Error {
    /// LSP 协议错误
    #[error("LSP protocol error: {0}")]
    Protocol(#[from] lsp_server::ProtocolError),

    /// TOML 解析错误
    #[error("TOML parse error in {uri}: {message}")]
    TomlParse {
        uri: String,
        message: String,
    },

    /// Rust 语法解析错误
    #[error("Rust parse error in {uri}: {message}")]
    RustParse {
        uri: String,
        message: String,
    },

    /// Schema 加载错误
    #[error("Schema load error: {0}")]
    SchemaLoad(String),

    /// 文件 I/O 错误
    #[error("File I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON 序列化/反序列化错误
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// HTTP 请求错误
    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),

    /// 其他错误
    #[error("{0}")]
    Other(#[from] anyhow::Error),
}

/// Result 类型别名
pub type Result<T> = std::result::Result<T, Error>;
