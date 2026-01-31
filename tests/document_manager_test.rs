//! 文档管理器集成测试
//!
//! 本测试文件包含文档管理器的集成测试和属性测试，验证：
//! - 文档打开和缓存（Requirements 1.3）
//! - 增量更新（Requirements 1.4）
//! - 文档关闭和缓存清理（Requirements 1.5）

use lsp_types::{Position, Range, TextDocumentContentChangeEvent, Url};
use proptest::prelude::*;
use spring_lsp::document::DocumentManager;

// ============================================================================
// 单元测试 - 验证具体示例和边缘情况
// ============================================================================

#[test]
fn test_document_open_and_cache() {
    let manager = DocumentManager::new();
    let uri: Url = "file:///test.toml".parse().unwrap();

    // 打开文档
    manager.open(
        uri.clone(),
        1,
        "host = \"localhost\"".to_string(),
        "toml".to_string(),
    );

    // 验证文档已缓存
    let doc = manager.get(&uri).expect("Document should be cached");
    assert_eq!(doc.uri, uri);
    assert_eq!(doc.version, 1);
    assert_eq!(doc.content, "host = \"localhost\"");
    assert_eq!(doc.language_id, "toml");
}

#[test]
fn test_document_open_empty_content() {
    let manager = DocumentManager::new();
    let uri: Url = "file:///empty.toml".parse().unwrap();

    // 打开空文档
    manager.open(uri.clone(), 1, String::new(), "toml".to_string());

    // 验证空文档已缓存
    let doc = manager.get(&uri).expect("Empty document should be cached");
    assert_eq!(doc.content, "");
}

#[test]
fn test_document_open_large_content() {
    let manager = DocumentManager::new();
    let uri: Url = "file:///large.toml".parse().unwrap();

    // 创建大文档（10000 行）
    let large_content = (0..10000)
        .map(|i| format!("key{} = \"value{}\"", i, i))
        .collect::<Vec<_>>()
        .join("\n");

    manager.open(uri.clone(), 1, large_content.clone(), "toml".to_string());

    // 验证大文档已缓存
    let doc = manager.get(&uri).expect("Large document should be cached");
    assert_eq!(doc.content, large_content);
}

#[test]
fn test_document_open_multiple_documents() {
    let manager = DocumentManager::new();

    // 打开多个文档
    for i in 0..10 {
        let uri: Url = format!("file:///test{}.toml", i).parse().unwrap();
        manager.open(
            uri.clone(),
            1,
            format!("content {}", i),
            "toml".to_string(),
        );
    }

    // 验证所有文档都已缓存
    for i in 0..10 {
        let uri: Url = format!("file:///test{}.toml", i).parse().unwrap();
        let doc = manager.get(&uri).expect("Document should be cached");
        assert_eq!(doc.content, format!("content {}", i));
    }
}

#[test]
fn test_document_reopen_updates_content() {
    let manager = DocumentManager::new();
    let uri: Url = "file:///test.toml".parse().unwrap();

    // 第一次打开
    manager.open(uri.clone(), 1, "old content".to_string(), "toml".to_string());
    let doc1 = manager.get(&uri).unwrap();
    assert_eq!(doc1.content, "old content");

    // 重新打开（模拟编辑器重新加载文件）
    manager.open(uri.clone(), 2, "new content".to_string(), "toml".to_string());
    let doc2 = manager.get(&uri).unwrap();
    assert_eq!(doc2.content, "new content");
    assert_eq!(doc2.version, 2);
}

#[test]
fn test_incremental_update_single_line_replace() {
    let manager = DocumentManager::new();
    let uri: Url = "file:///test.toml".parse().unwrap();

    manager.open(uri.clone(), 1, "hello world".to_string(), "toml".to_string());

    // 替换 "world" 为 "rust"
    let changes = vec![TextDocumentContentChangeEvent {
        range: Some(Range {
            start: Position {
                line: 0,
                character: 6,
            },
            end: Position {
                line: 0,
                character: 11,
            },
        }),
        range_length: None,
        text: "rust".to_string(),
    }];

    manager.change(&uri, 2, changes);

    let doc = manager.get(&uri).unwrap();
    assert_eq!(doc.version, 2);
    assert_eq!(doc.content, "hello rust");
}

#[test]
fn test_incremental_update_multiline_replace() {
    let manager = DocumentManager::new();
    let uri: Url = "file:///test.toml".parse().unwrap();

    let initial = "line 1\nline 2\nline 3";
    manager.open(uri.clone(), 1, initial.to_string(), "toml".to_string());

    // 替换第二行
    let changes = vec![TextDocumentContentChangeEvent {
        range: Some(Range {
            start: Position {
                line: 1,
                character: 0,
            },
            end: Position {
                line: 1,
                character: 6,
            },
        }),
        range_length: None,
        text: "modified".to_string(),
    }];

    manager.change(&uri, 2, changes);

    let doc = manager.get(&uri).unwrap();
    assert_eq!(doc.content, "line 1\nmodified\nline 3");
}

#[test]
fn test_incremental_update_insert_at_beginning() {
    let manager = DocumentManager::new();
    let uri: Url = "file:///test.toml".parse().unwrap();

    manager.open(uri.clone(), 1, "world".to_string(), "toml".to_string());

    // 在开头插入
    let changes = vec![TextDocumentContentChangeEvent {
        range: Some(Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: 0,
                character: 0,
            },
        }),
        range_length: None,
        text: "hello ".to_string(),
    }];

    manager.change(&uri, 2, changes);

    let doc = manager.get(&uri).unwrap();
    assert_eq!(doc.content, "hello world");
}

#[test]
fn test_incremental_update_insert_at_end() {
    let manager = DocumentManager::new();
    let uri: Url = "file:///test.toml".parse().unwrap();

    manager.open(uri.clone(), 1, "hello".to_string(), "toml".to_string());

    // 在末尾插入
    let changes = vec![TextDocumentContentChangeEvent {
        range: Some(Range {
            start: Position {
                line: 0,
                character: 5,
            },
            end: Position {
                line: 0,
                character: 5,
            },
        }),
        range_length: None,
        text: " world".to_string(),
    }];

    manager.change(&uri, 2, changes);

    let doc = manager.get(&uri).unwrap();
    assert_eq!(doc.content, "hello world");
}

#[test]
fn test_incremental_update_delete() {
    let manager = DocumentManager::new();
    let uri: Url = "file:///test.toml".parse().unwrap();

    manager.open(
        uri.clone(),
        1,
        "hello world".to_string(),
        "toml".to_string(),
    );

    // 删除 " world"
    let changes = vec![TextDocumentContentChangeEvent {
        range: Some(Range {
            start: Position {
                line: 0,
                character: 5,
            },
            end: Position {
                line: 0,
                character: 11,
            },
        }),
        range_length: None,
        text: String::new(),
    }];

    manager.change(&uri, 2, changes);

    let doc = manager.get(&uri).unwrap();
    assert_eq!(doc.content, "hello");
}

#[test]
fn test_incremental_update_delete_entire_line() {
    let manager = DocumentManager::new();
    let uri: Url = "file:///test.toml".parse().unwrap();

    let initial = "line 1\nline 2\nline 3";
    manager.open(uri.clone(), 1, initial.to_string(), "toml".to_string());

    // 删除第二行（包括换行符）
    let changes = vec![TextDocumentContentChangeEvent {
        range: Some(Range {
            start: Position {
                line: 1,
                character: 0,
            },
            end: Position {
                line: 2,
                character: 0,
            },
        }),
        range_length: None,
        text: String::new(),
    }];

    manager.change(&uri, 2, changes);

    let doc = manager.get(&uri).unwrap();
    assert_eq!(doc.content, "line 1\nline 3");
}

#[test]
fn test_incremental_update_utf8_content() {
    let manager = DocumentManager::new();
    let uri: Url = "file:///test.toml".parse().unwrap();

    manager.open(uri.clone(), 1, "你好世界".to_string(), "toml".to_string());

    // 替换 "世界" 为 "Rust"
    let changes = vec![TextDocumentContentChangeEvent {
        range: Some(Range {
            start: Position {
                line: 0,
                character: 2,
            },
            end: Position {
                line: 0,
                character: 4,
            },
        }),
        range_length: None,
        text: "Rust".to_string(),
    }];

    manager.change(&uri, 2, changes);

    let doc = manager.get(&uri).unwrap();
    assert_eq!(doc.content, "你好Rust");
}

#[test]
fn test_incremental_update_emoji() {
    let manager = DocumentManager::new();
    let uri: Url = "file:///test.toml".parse().unwrap();

    manager.open(uri.clone(), 1, "Hello 🦀".to_string(), "toml".to_string());

    // 在 emoji 后插入文本
    let changes = vec![TextDocumentContentChangeEvent {
        range: Some(Range {
            start: Position {
                line: 0,
                character: 7,
            },
            end: Position {
                line: 0,
                character: 7,
            },
        }),
        range_length: None,
        text: " Rust".to_string(),
    }];

    manager.change(&uri, 2, changes);

    let doc = manager.get(&uri).unwrap();
    assert_eq!(doc.content, "Hello 🦀 Rust");
}

#[test]
fn test_full_content_update() {
    let manager = DocumentManager::new();
    let uri: Url = "file:///test.toml".parse().unwrap();

    manager.open(uri.clone(), 1, "old content".to_string(), "toml".to_string());

    // 全量更新（range 为 None）
    let changes = vec![TextDocumentContentChangeEvent {
        range: None,
        range_length: None,
        text: "completely new content".to_string(),
    }];

    manager.change(&uri, 2, changes);

    let doc = manager.get(&uri).unwrap();
    assert_eq!(doc.version, 2);
    assert_eq!(doc.content, "completely new content");
}

#[test]
fn test_multiple_sequential_changes() {
    let manager = DocumentManager::new();
    let uri: Url = "file:///test.toml".parse().unwrap();

    manager.open(uri.clone(), 1, "a b c".to_string(), "toml".to_string());

    // 第一次修改
    let changes1 = vec![TextDocumentContentChangeEvent {
        range: Some(Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: 0,
                character: 1,
            },
        }),
        range_length: None,
        text: "x".to_string(),
    }];
    manager.change(&uri, 2, changes1);

    // 第二次修改
    let changes2 = vec![TextDocumentContentChangeEvent {
        range: Some(Range {
            start: Position {
                line: 0,
                character: 2,
            },
            end: Position {
                line: 0,
                character: 3,
            },
        }),
        range_length: None,
        text: "y".to_string(),
    }];
    manager.change(&uri, 3, changes2);

    let doc = manager.get(&uri).unwrap();
    assert_eq!(doc.version, 3);
    assert_eq!(doc.content, "x y c");
}

#[test]
fn test_document_close_removes_from_cache() {
    let manager = DocumentManager::new();
    let uri: Url = "file:///test.toml".parse().unwrap();

    // 打开文档
    manager.open(uri.clone(), 1, "content".to_string(), "toml".to_string());
    assert!(manager.get(&uri).is_some());

    // 关闭文档
    manager.close(&uri);

    // 验证文档已从缓存中移除
    assert!(manager.get(&uri).is_none());
}

#[test]
fn test_document_close_nonexistent() {
    let manager = DocumentManager::new();
    let uri: Url = "file:///nonexistent.toml".parse().unwrap();

    // 关闭不存在的文档不应该崩溃
    manager.close(&uri);
}

#[test]
fn test_document_close_multiple_documents() {
    let manager = DocumentManager::new();

    // 打开多个文档
    let uris: Vec<Url> = (0..5)
        .map(|i| format!("file:///test{}.toml", i).parse().unwrap())
        .collect();

    for (i, uri) in uris.iter().enumerate() {
        manager.open(
            uri.clone(),
            1,
            format!("content {}", i),
            "toml".to_string(),
        );
    }

    // 关闭部分文档
    manager.close(&uris[1]);
    manager.close(&uris[3]);

    // 验证正确的文档被移除
    assert!(manager.get(&uris[0]).is_some());
    assert!(manager.get(&uris[1]).is_none());
    assert!(manager.get(&uris[2]).is_some());
    assert!(manager.get(&uris[3]).is_none());
    assert!(manager.get(&uris[4]).is_some());
}

#[test]
fn test_with_document_callback() {
    let manager = DocumentManager::new();
    let uri: Url = "file:///test.toml".parse().unwrap();

    manager.open(
        uri.clone(),
        1,
        "hello world".to_string(),
        "toml".to_string(),
    );

    // 使用回调访问文档
    let length = manager.with_document(&uri, |doc| doc.content.len());
    assert_eq!(length, Some(11));

    let version = manager.with_document(&uri, |doc| doc.version);
    assert_eq!(version, Some(1));

    // 访问不存在的文档
    let nonexistent: Url = "file:///nonexistent.toml".parse().unwrap();
    let result = manager.with_document(&nonexistent, |doc| doc.content.len());
    assert_eq!(result, None);
}

#[test]
fn test_cache_consistency_after_operations() {
    let manager = DocumentManager::new();
    let uri: Url = "file:///test.toml".parse().unwrap();

    // 打开
    manager.open(uri.clone(), 1, "initial".to_string(), "toml".to_string());
    let doc1 = manager.get(&uri).unwrap();
    assert_eq!(doc1.content, "initial");

    // 修改
    let changes = vec![TextDocumentContentChangeEvent {
        range: None,
        range_length: None,
        text: "modified".to_string(),
    }];
    manager.change(&uri, 2, changes);
    let doc2 = manager.get(&uri).unwrap();
    assert_eq!(doc2.content, "modified");
    assert_eq!(doc2.version, 2);

    // 再次修改
    let changes = vec![TextDocumentContentChangeEvent {
        range: None,
        range_length: None,
        text: "final".to_string(),
    }];
    manager.change(&uri, 3, changes);
    let doc3 = manager.get(&uri).unwrap();
    assert_eq!(doc3.content, "final");
    assert_eq!(doc3.version, 3);

    // 关闭
    manager.close(&uri);
    assert!(manager.get(&uri).is_none());
}

// ============================================================================
// 属性测试 - 验证通用属性在所有输入下的正确性
// ============================================================================

/// 生成有效的文档内容
fn arb_document_content() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-zA-Z0-9 \n\t=\":_-]*").unwrap()
}

/// 生成有效的 URI
fn arb_uri() -> impl Strategy<Value = Url> {
    prop::string::string_regex("file:///[a-z0-9_-]+\\.toml")
        .unwrap()
        .prop_map(|s| s.parse::<Url>().unwrap())
}

/// 生成有效的版本号
fn arb_version() -> impl Strategy<Value = i32> {
    1i32..1000
}

/// 生成有效的语言 ID
fn arb_language_id() -> impl Strategy<Value = String> {
    prop::sample::select(vec!["toml", "rust", "json", "yaml"]).prop_map(|s| s.to_string())
}

// Feature: spring-lsp, Property 2: 文档缓存一致性
// **Validates: Requirements 1.3**
proptest! {
    #[test]
    fn prop_document_cache_consistency(
        uri in arb_uri(),
        version in arb_version(),
        content in arb_document_content(),
        language_id in arb_language_id(),
    ) {
        let manager = DocumentManager::new();

        // 打开文档
        manager.open(uri.clone(), version, content.clone(), language_id.clone());

        // 验证缓存的文档内容与输入完全一致
        let cached = manager.get(&uri).expect("Document should be cached");
        prop_assert_eq!(cached.uri, uri);
        prop_assert_eq!(cached.version, version);
        prop_assert_eq!(cached.content, content);
        prop_assert_eq!(cached.language_id, language_id);
    }
}

// Feature: spring-lsp, Property 3: 增量更新正确性
// **Validates: Requirements 1.4**
proptest! {
    #[test]
    fn prop_full_content_change_correctness(
        uri in arb_uri(),
        initial_content in arb_document_content(),
        new_content in arb_document_content(),
    ) {
        let manager = DocumentManager::new();

        // 打开文档
        manager.open(uri.clone(), 1, initial_content, "toml".to_string());

        // 全量更新
        let changes = vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: new_content.clone(),
        }];
        manager.change(&uri, 2, changes);

        // 验证更新后的内容正确
        let doc = manager.get(&uri).expect("Document should exist");
        prop_assert_eq!(doc.content, new_content);
        prop_assert_eq!(doc.version, 2);
    }
}

// Feature: spring-lsp, Property 4: 缓存清理完整性
// **Validates: Requirements 1.5**
proptest! {
    #[test]
    fn prop_cache_cleanup_completeness(
        uri in arb_uri(),
        content in arb_document_content(),
    ) {
        let manager = DocumentManager::new();

        // 打开文档
        manager.open(uri.clone(), 1, content, "toml".to_string());
        prop_assert!(manager.get(&uri).is_some());

        // 关闭文档
        manager.close(&uri);

        // 验证文档已完全从缓存中移除
        prop_assert!(manager.get(&uri).is_none());
    }
}

// 属性测试：多文档独立性
proptest! {
    #[test]
    fn prop_multiple_documents_independence(
        uris in prop::collection::vec(arb_uri(), 1..10),
        contents in prop::collection::vec(arb_document_content(), 1..10),
    ) {
        let manager = DocumentManager::new();
        let count = uris.len().min(contents.len());

        // 打开多个文档（使用 HashMap 跟踪最后一次打开的内容，因为重复的 URI 会覆盖）
        use std::collections::HashMap;
        let mut expected: HashMap<Url, String> = HashMap::new();
        
        for i in 0..count {
            manager.open(
                uris[i].clone(),
                1,
                contents[i].clone(),
                "toml".to_string(),
            );
            // 记录最后一次打开的内容
            expected.insert(uris[i].clone(), contents[i].clone());
        }

        // 验证每个唯一 URI 的内容正确
        for (uri, expected_content) in expected.iter() {
            let doc = manager.get(uri).expect("Document should exist");
            prop_assert_eq!(&doc.content, expected_content);
        }
    }
}

// 属性测试：版本号单调递增
proptest! {
    #[test]
    fn prop_version_monotonic_increase(
        uri in arb_uri(),
        initial_content in arb_document_content(),
        changes in prop::collection::vec(arb_document_content(), 1..10),
    ) {
        let manager = DocumentManager::new();

        // 打开文档
        manager.open(uri.clone(), 1, initial_content, "toml".to_string());

        // 应用多次修改
        for (i, new_content) in changes.iter().enumerate() {
            let version = (i + 2) as i32;
            let change = vec![TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text: new_content.clone(),
            }];
            manager.change(&uri, version, change);

            // 验证版本号正确更新
            let doc = manager.get(&uri).expect("Document should exist");
            prop_assert_eq!(doc.version, version);
        }
    }
}

// 属性测试：关闭不存在的文档不会崩溃
proptest! {
    #[test]
    fn prop_close_nonexistent_safe(uri in arb_uri()) {
        let manager = DocumentManager::new();

        // 关闭不存在的文档不应该崩溃
        manager.close(&uri);

        // 验证仍然可以正常操作
        manager.open(uri.clone(), 1, "test".to_string(), "toml".to_string());
        prop_assert!(manager.get(&uri).is_some());
    }
}

// 属性测试：重复打开文档会覆盖旧内容
proptest! {
    #[test]
    fn prop_reopen_overwrites(
        uri in arb_uri(),
        content1 in arb_document_content(),
        content2 in arb_document_content(),
    ) {
        let manager = DocumentManager::new();

        // 第一次打开
        manager.open(uri.clone(), 1, content1, "toml".to_string());

        // 第二次打开（覆盖）
        manager.open(uri.clone(), 2, content2.clone(), "toml".to_string());

        // 验证内容被覆盖
        let doc = manager.get(&uri).expect("Document should exist");
        prop_assert_eq!(doc.content, content2);
        prop_assert_eq!(doc.version, 2);
    }
}

// 属性测试：with_document 回调的正确性
proptest! {
    #[test]
    fn prop_with_document_correctness(
        uri in arb_uri(),
        content in arb_document_content(),
    ) {
        let manager = DocumentManager::new();

        manager.open(uri.clone(), 1, content.clone(), "toml".to_string());

        // 使用 with_document 访问
        let result = manager.with_document(&uri, |doc| {
            (doc.content.clone(), doc.version)
        });

        prop_assert!(result.is_some());
        let (cached_content, cached_version) = result.unwrap();
        prop_assert_eq!(cached_content, content);
        prop_assert_eq!(cached_version, 1);
    }
}
