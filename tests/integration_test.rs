//! 集成测试

use spring_lsp::document::DocumentManager;
use spring_lsp::diagnostic::DiagnosticEngine;

#[test]
fn test_document_manager_creation() {
    let manager = DocumentManager::new();
    // 验证文档管理器可以创建
    assert!(manager.get(&"file:///test.toml".parse().unwrap()).is_none());
}

#[test]
fn test_diagnostic_engine_creation() {
    let engine = DiagnosticEngine::new();
    // 验证诊断引擎可以创建
    assert!(engine.get(&"file:///test.toml".parse().unwrap()).is_empty());
}
