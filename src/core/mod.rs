//! 核心功能模块
//!
//! 提供文档管理、索引、Schema 等核心功能

pub mod document;
pub mod index;
pub mod schema;
pub mod config;

pub use document::DocumentManager;
pub use index::SymbolIndex;
pub use schema::SchemaManager;
pub use config::ConfigManager;
