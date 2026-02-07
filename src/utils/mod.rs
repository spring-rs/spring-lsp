//! 工具和辅助模块
//!
//! 提供错误处理、日志、状态管理等工具

pub mod error;
pub mod logging;
pub mod status;

pub use error::{Error, Result};
pub use logging::init_logging;
pub use status::ServerStatus;
