//! LSP 协议类型定义
//!
//! 定义 LSP 通信中使用的自定义类型

use serde::{Deserialize, Serialize};

/// 位置信息响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationResponse {
    /// 文件 URI
    pub uri: String,
    /// 范围
    pub range: RangeResponse,
}

/// 范围响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RangeResponse {
    /// 起始位置
    pub start: PositionResponse,
    /// 结束位置
    pub end: PositionResponse,
}

/// 位置响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionResponse {
    /// 行号（从 0 开始）
    pub line: u32,
    /// 列号（从 0 开始）
    pub character: u32,
}
