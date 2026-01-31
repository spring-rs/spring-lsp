//! 诊断引擎模块

use dashmap::DashMap;
use lsp_types::{Diagnostic, Url};

/// 诊断引擎
pub struct DiagnosticEngine {
    /// 诊断缓存（DashMap 本身就是并发安全的）
    diagnostics: DashMap<Url, Vec<Diagnostic>>,
}

impl DiagnosticEngine {
    /// 创建新的诊断引擎
    pub fn new() -> Self {
        Self {
            diagnostics: DashMap::new(),
        }
    }

    /// 添加诊断
    pub fn add(&self, uri: Url, diagnostic: Diagnostic) {
        self.diagnostics
            .entry(uri)
            .or_insert_with(Vec::new)
            .push(diagnostic);
    }

    /// 清除文档的诊断
    pub fn clear(&self, uri: &Url) {
        self.diagnostics.remove(uri);
    }

    /// 获取文档的诊断（返回克隆）
    pub fn get(&self, uri: &Url) -> Vec<Diagnostic> {
        self.diagnostics
            .get(uri)
            .map(|v| v.clone())
            .unwrap_or_default()
    }
}

impl Default for DiagnosticEngine {
    fn default() -> Self {
        Self::new()
    }
}
