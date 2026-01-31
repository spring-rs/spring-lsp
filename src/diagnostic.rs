//! 诊断引擎模块

use dashmap::DashMap;
use lsp_server::Connection;
use lsp_types::{Diagnostic, PublishDiagnosticsParams, Url};

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

    /// 发布诊断到客户端
    ///
    /// 通过 LSP 的 `textDocument/publishDiagnostics` 通知将诊断信息发送给客户端。
    /// 如果文档没有诊断信息，将发送空的诊断列表以清除之前的诊断。
    ///
    /// # 参数
    ///
    /// * `connection` - LSP 连接，用于发送通知
    /// * `uri` - 文档 URI
    ///
    /// # 返回
    ///
    /// 如果发送成功返回 `Ok(())`，否则返回错误
    pub fn publish(&self, connection: &Connection, uri: &Url) -> crate::Result<()> {
        use lsp_server::{Message, Notification};
        use lsp_types::notification::{Notification as _, PublishDiagnostics};

        // 获取文档的诊断（如果没有则为空列表）
        let diagnostics = self.get(uri);
        let diagnostics_count = diagnostics.len();

        // 创建发布诊断参数
        let params = PublishDiagnosticsParams {
            uri: uri.clone(),
            diagnostics,
            version: None,
        };

        // 创建通知
        let notification = Notification {
            method: PublishDiagnostics::METHOD.to_string(),
            params: serde_json::to_value(params)
                .map_err(|e| crate::Error::Other(anyhow::anyhow!("Failed to serialize diagnostics: {}", e)))?,
        };

        // 发送通知
        connection
            .sender
            .send(Message::Notification(notification))
            .map_err(|e| crate::Error::Other(anyhow::anyhow!("Failed to send diagnostics: {}", e)))?;

        tracing::debug!("Published {} diagnostics for {}", diagnostics_count, uri);

        Ok(())
    }
}

impl Default for DiagnosticEngine {
    fn default() -> Self {
        Self::new()
    }
}
