//! 代码分析模块
//!
//! 提供各种代码分析功能

pub mod toml;
pub mod rust;
pub mod completion;
pub mod diagnostic;
pub mod validation;

pub use completion::CompletionEngine;
pub use diagnostic::DiagnosticEngine;
