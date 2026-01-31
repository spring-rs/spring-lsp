//! DiagnosticEngine 单元测试
//!
//! 测试诊断引擎的核心功能：
//! - 添加诊断
//! - 清除诊断
//! - 获取诊断
//! - 发布诊断到客户端

use lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range, Url};
use spring_lsp::diagnostic::DiagnosticEngine;

/// 创建测试用的诊断
fn create_test_diagnostic(message: &str, severity: DiagnosticSeverity) -> Diagnostic {
    Diagnostic {
        range: Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: 0,
                character: 10,
            },
        },
        severity: Some(severity),
        code: None,
        code_description: None,
        source: Some("spring-lsp".to_string()),
        message: message.to_string(),
        related_information: None,
        tags: None,
        data: None,
    }
}

#[test]
fn test_diagnostic_engine_new() {
    let engine = DiagnosticEngine::new();
    let uri = Url::parse("file:///test.toml").unwrap();
    
    // 新创建的引擎不应该有任何诊断
    assert!(engine.get(&uri).is_empty());
}

#[test]
fn test_add_single_diagnostic() {
    let engine = DiagnosticEngine::new();
    let uri = Url::parse("file:///test.toml").unwrap();
    let diagnostic = create_test_diagnostic("Test error", DiagnosticSeverity::ERROR);
    
    // 添加诊断
    engine.add(uri.clone(), diagnostic.clone());
    
    // 验证诊断已添加
    let diagnostics = engine.get(&uri);
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].message, "Test error");
    assert_eq!(diagnostics[0].severity, Some(DiagnosticSeverity::ERROR));
}

#[test]
fn test_add_multiple_diagnostics() {
    let engine = DiagnosticEngine::new();
    let uri = Url::parse("file:///test.toml").unwrap();
    
    // 添加多个诊断
    engine.add(uri.clone(), create_test_diagnostic("Error 1", DiagnosticSeverity::ERROR));
    engine.add(uri.clone(), create_test_diagnostic("Warning 1", DiagnosticSeverity::WARNING));
    engine.add(uri.clone(), create_test_diagnostic("Info 1", DiagnosticSeverity::INFORMATION));
    
    // 验证所有诊断都已添加
    let diagnostics = engine.get(&uri);
    assert_eq!(diagnostics.len(), 3);
    assert_eq!(diagnostics[0].message, "Error 1");
    assert_eq!(diagnostics[1].message, "Warning 1");
    assert_eq!(diagnostics[2].message, "Info 1");
}

#[test]
fn test_add_diagnostics_to_different_files() {
    let engine = DiagnosticEngine::new();
    let uri1 = Url::parse("file:///test1.toml").unwrap();
    let uri2 = Url::parse("file:///test2.toml").unwrap();
    
    // 为不同文件添加诊断
    engine.add(uri1.clone(), create_test_diagnostic("Error in file 1", DiagnosticSeverity::ERROR));
    engine.add(uri2.clone(), create_test_diagnostic("Error in file 2", DiagnosticSeverity::ERROR));
    
    // 验证每个文件都有自己的诊断
    let diagnostics1 = engine.get(&uri1);
    let diagnostics2 = engine.get(&uri2);
    
    assert_eq!(diagnostics1.len(), 1);
    assert_eq!(diagnostics1[0].message, "Error in file 1");
    
    assert_eq!(diagnostics2.len(), 1);
    assert_eq!(diagnostics2[0].message, "Error in file 2");
}

#[test]
fn test_clear_diagnostics() {
    let engine = DiagnosticEngine::new();
    let uri = Url::parse("file:///test.toml").unwrap();
    
    // 添加诊断
    engine.add(uri.clone(), create_test_diagnostic("Error", DiagnosticSeverity::ERROR));
    engine.add(uri.clone(), create_test_diagnostic("Warning", DiagnosticSeverity::WARNING));
    
    // 验证诊断已添加
    assert_eq!(engine.get(&uri).len(), 2);
    
    // 清除诊断
    engine.clear(&uri);
    
    // 验证诊断已清除
    assert!(engine.get(&uri).is_empty());
}

#[test]
fn test_clear_nonexistent_file() {
    let engine = DiagnosticEngine::new();
    let uri = Url::parse("file:///nonexistent.toml").unwrap();
    
    // 清除不存在的文件的诊断（不应该崩溃）
    engine.clear(&uri);
    
    // 验证仍然没有诊断
    assert!(engine.get(&uri).is_empty());
}

#[test]
fn test_clear_does_not_affect_other_files() {
    let engine = DiagnosticEngine::new();
    let uri1 = Url::parse("file:///test1.toml").unwrap();
    let uri2 = Url::parse("file:///test2.toml").unwrap();
    
    // 为两个文件添加诊断
    engine.add(uri1.clone(), create_test_diagnostic("Error 1", DiagnosticSeverity::ERROR));
    engine.add(uri2.clone(), create_test_diagnostic("Error 2", DiagnosticSeverity::ERROR));
    
    // 清除第一个文件的诊断
    engine.clear(&uri1);
    
    // 验证第一个文件的诊断已清除，第二个文件的诊断仍然存在
    assert!(engine.get(&uri1).is_empty());
    assert_eq!(engine.get(&uri2).len(), 1);
}

#[test]
fn test_get_returns_clone() {
    let engine = DiagnosticEngine::new();
    let uri = Url::parse("file:///test.toml").unwrap();
    
    // 添加诊断
    engine.add(uri.clone(), create_test_diagnostic("Error", DiagnosticSeverity::ERROR));
    
    // 获取诊断两次
    let diagnostics1 = engine.get(&uri);
    let diagnostics2 = engine.get(&uri);
    
    // 验证两次获取的结果相同
    assert_eq!(diagnostics1.len(), diagnostics2.len());
    assert_eq!(diagnostics1[0].message, diagnostics2[0].message);
}

#[test]
fn test_get_empty_for_nonexistent_file() {
    let engine = DiagnosticEngine::new();
    let uri = Url::parse("file:///nonexistent.toml").unwrap();
    
    // 获取不存在的文件的诊断
    let diagnostics = engine.get(&uri);
    
    // 应该返回空列表
    assert!(diagnostics.is_empty());
}

#[test]
fn test_diagnostic_severity_levels() {
    let engine = DiagnosticEngine::new();
    let uri = Url::parse("file:///test.toml").unwrap();
    
    // 添加不同严重级别的诊断
    engine.add(uri.clone(), create_test_diagnostic("Error", DiagnosticSeverity::ERROR));
    engine.add(uri.clone(), create_test_diagnostic("Warning", DiagnosticSeverity::WARNING));
    engine.add(uri.clone(), create_test_diagnostic("Info", DiagnosticSeverity::INFORMATION));
    engine.add(uri.clone(), create_test_diagnostic("Hint", DiagnosticSeverity::HINT));
    
    // 验证所有严重级别都被正确存储
    let diagnostics = engine.get(&uri);
    assert_eq!(diagnostics.len(), 4);
    assert_eq!(diagnostics[0].severity, Some(DiagnosticSeverity::ERROR));
    assert_eq!(diagnostics[1].severity, Some(DiagnosticSeverity::WARNING));
    assert_eq!(diagnostics[2].severity, Some(DiagnosticSeverity::INFORMATION));
    assert_eq!(diagnostics[3].severity, Some(DiagnosticSeverity::HINT));
}

#[test]
fn test_diagnostic_with_code() {
    let engine = DiagnosticEngine::new();
    let uri = Url::parse("file:///test.toml").unwrap();
    
    let mut diagnostic = create_test_diagnostic("Error with code", DiagnosticSeverity::ERROR);
    diagnostic.code = Some(lsp_types::NumberOrString::String("E001".to_string()));
    
    engine.add(uri.clone(), diagnostic);
    
    let diagnostics = engine.get(&uri);
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].code,
        Some(lsp_types::NumberOrString::String("E001".to_string()))
    );
}

#[test]
fn test_diagnostic_with_source() {
    let engine = DiagnosticEngine::new();
    let uri = Url::parse("file:///test.toml").unwrap();
    
    let diagnostic = create_test_diagnostic("Error from spring-lsp", DiagnosticSeverity::ERROR);
    
    engine.add(uri.clone(), diagnostic);
    
    let diagnostics = engine.get(&uri);
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].source, Some("spring-lsp".to_string()));
}

#[test]
fn test_concurrent_access() {
    use std::sync::Arc;
    use std::thread;
    
    let engine = Arc::new(DiagnosticEngine::new());
    let uri = Url::parse("file:///test.toml").unwrap();
    
    // 创建多个线程同时添加诊断
    let mut handles = vec![];
    
    for i in 0..10 {
        let engine_clone = Arc::clone(&engine);
        let uri_clone = uri.clone();
        
        let handle = thread::spawn(move || {
            engine_clone.add(
                uri_clone,
                create_test_diagnostic(
                    &format!("Error {}", i),
                    DiagnosticSeverity::ERROR,
                ),
            );
        });
        
        handles.push(handle);
    }
    
    // 等待所有线程完成
    for handle in handles {
        handle.join().unwrap();
    }
    
    // 验证所有诊断都已添加
    let diagnostics = engine.get(&uri);
    assert_eq!(diagnostics.len(), 10);
}

#[test]
fn test_default_trait() {
    let engine = DiagnosticEngine::default();
    let uri = Url::parse("file:///test.toml").unwrap();
    
    // 验证 default() 创建的引擎与 new() 创建的引擎行为相同
    assert!(engine.get(&uri).is_empty());
}

// 注意：publish() 方法的测试需要模拟 LSP 连接，这在单元测试中比较困难
// 我们将在集成测试中测试 publish() 方法
// 这里我们只测试 publish() 方法不会崩溃（如果可以创建测试连接的话）

#[test]
fn test_add_and_clear_cycle() {
    let engine = DiagnosticEngine::new();
    let uri = Url::parse("file:///test.toml").unwrap();
    
    // 多次添加和清除诊断
    for i in 0..5 {
        engine.add(
            uri.clone(),
            create_test_diagnostic(&format!("Error {}", i), DiagnosticSeverity::ERROR),
        );
        assert_eq!(engine.get(&uri).len(), 1);
        
        engine.clear(&uri);
        assert!(engine.get(&uri).is_empty());
    }
}

#[test]
fn test_diagnostic_range() {
    let engine = DiagnosticEngine::new();
    let uri = Url::parse("file:///test.toml").unwrap();
    
    let mut diagnostic = create_test_diagnostic("Error", DiagnosticSeverity::ERROR);
    diagnostic.range = Range {
        start: Position {
            line: 5,
            character: 10,
        },
        end: Position {
            line: 5,
            character: 20,
        },
    };
    
    engine.add(uri.clone(), diagnostic);
    
    let diagnostics = engine.get(&uri);
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].range.start.line, 5);
    assert_eq!(diagnostics[0].range.start.character, 10);
    assert_eq!(diagnostics[0].range.end.line, 5);
    assert_eq!(diagnostics[0].range.end.character, 20);
}
