//! # spring-lsp
//!
//! Language Server Protocol implementation for spring-rs framework.
//!
//! spring-lsp 提供智能的开发体验，包括：
//! - TOML 配置文件的智能补全和验证
//! - Rust 宏的分析和展开
//! - 路由的识别和导航
//! - 依赖注入验证
//!
//! ## 架构
//!
//! spring-lsp 采用分层架构：
//! - **LSP Protocol Layer**: 处理 LSP 协议通信
//! - **Server Core Layer**: 消息分发和状态管理
//! - **Analysis Modules**: 各种分析功能模块
//! - **Foundation Layer**: 基础设施和工具

pub mod completion;
pub mod component_scanner;
pub mod config;
pub mod config_scanner;
pub mod di_validator;
pub mod diagnostic;
pub mod document;
pub mod error;
pub mod index;
pub mod job_scanner;
pub mod logging;
pub mod macro_analyzer;
pub mod plugin_scanner;
pub mod route;
pub mod route_scanner;
pub mod schema;
pub mod server;
pub mod status;
pub mod toml_analyzer;

pub use error::{Error, Result};
