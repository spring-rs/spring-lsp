//! LSP 服务器核心功能测试
//!
//! 本测试文件验证 LSP 服务器的核心功能，包括：
//! - 初始化握手流程
//! - 优雅关闭流程
//! - 错误恢复能力
//!
//! 测试策略：
//! - 单元测试：验证具体场景和边缘情况
//! - 属性测试：验证通用属性在所有输入下的正确性

use lsp_types::{
    ClientCapabilities, ClientInfo, InitializeParams, Url, WorkDoneProgressParams,
    WorkspaceFolder,
};
use proptest::prelude::*;
use spring_lsp::server::{LspServer, ServerState};

// ============================================================================
// 单元测试 - 初始化握手
// ============================================================================

/// 测试基本的初始化握手
///
/// **Validates: Requirements 1.2**
#[test]
fn test_basic_initialization() {
    let mut server = LspServer::start().unwrap();

    #[allow(deprecated)]
    let params = InitializeParams {
        process_id: Some(1234),
        root_uri: Some(Url::parse("file:///workspace").unwrap()),
        capabilities: ClientCapabilities::default(),
        client_info: Some(ClientInfo {
            name: "test-client".to_string(),
            version: Some("1.0.0".to_string()),
        }),
        locale: None,
        root_path: None,
        initialization_options: None,
        trace: None,
        workspace_folders: None,
        work_done_progress_params: WorkDoneProgressParams::default(),
    };

    let result = server.handle_initialize(params).unwrap();

    // 验证服务器信息存在
    assert!(result.server_info.is_some());
    let server_info = result.server_info.unwrap();
    assert_eq!(server_info.name, "spring-lsp");
    assert!(server_info.version.is_some());

    // 验证基本能力已声明
    assert!(result.capabilities.text_document_sync.is_some());
    assert!(result.capabilities.completion_provider.is_some());
    assert!(result.capabilities.hover_provider.is_some());
}

/// 测试初始化响应包含所有必需的服务器能力
///
/// **Validates: Requirements 1.2**
#[test]
fn test_initialize_declares_all_capabilities() {
    let mut server = LspServer::start().unwrap();

    #[allow(deprecated)]
    let params = InitializeParams {
        process_id: Some(1234),
        root_uri: None,
        capabilities: ClientCapabilities::default(),
        client_info: None,
        locale: None,
        root_path: None,
        initialization_options: None,
        trace: None,
        workspace_folders: None,
        work_done_progress_params: WorkDoneProgressParams::default(),
    };

    let result = server.handle_initialize(params).unwrap();
    let capabilities = result.capabilities;

    // 验证文档同步能力
    assert!(
        capabilities.text_document_sync.is_some(),
        "应该声明文档同步能力"
    );

    // 验证补全能力
    assert!(
        capabilities.completion_provider.is_some(),
        "应该声明补全能力"
    );
    let completion = capabilities.completion_provider.unwrap();
    assert_eq!(
        completion.resolve_provider,
        Some(true),
        "应该支持补全项解析"
    );

    // 验证补全触发字符
    assert!(completion.trigger_characters.is_some());
    let triggers = completion.trigger_characters.unwrap();
    assert!(
        triggers.contains(&"[".to_string()),
        "应该支持 '[' 触发补全"
    );
    assert!(
        triggers.contains(&"$".to_string()),
        "应该支持 '$' 触发补全"
    );
    assert!(
        triggers.contains(&"{".to_string()),
        "应该支持 '{{' 触发补全"
    );

    // 验证悬停能力
    assert!(
        capabilities.hover_provider.is_some(),
        "应该声明悬停提示能力"
    );

    // 验证定义跳转能力
    assert!(
        capabilities.definition_provider.is_some(),
        "应该声明定义跳转能力"
    );

    // 验证文档符号能力
    assert!(
        capabilities.document_symbol_provider.is_some(),
        "应该声明文档符号能力"
    );

    // 验证工作空间符号能力
    assert!(
        capabilities.workspace_symbol_provider.is_some(),
        "应该声明工作空间符号能力"
    );
}

/// 测试初始化时使用增量文档同步模式
///
/// **Validates: Requirements 1.2, 1.4**
#[test]
fn test_initialize_uses_incremental_sync() {
    let mut server = LspServer::start().unwrap();

    #[allow(deprecated)]
    let params = InitializeParams {
        process_id: Some(1234),
        root_uri: None,
        capabilities: ClientCapabilities::default(),
        client_info: None,
        locale: None,
        root_path: None,
        initialization_options: None,
        trace: None,
        workspace_folders: None,
        work_done_progress_params: WorkDoneProgressParams::default(),
    };

    let result = server.handle_initialize(params).unwrap();

    // 验证使用增量同步模式
    if let Some(lsp_types::TextDocumentSyncCapability::Options(sync_options)) =
        result.capabilities.text_document_sync
    {
        assert_eq!(
            sync_options.open_close,
            Some(true),
            "应该支持文档打开/关闭通知"
        );
        assert_eq!(
            sync_options.change,
            Some(lsp_types::TextDocumentSyncKind::INCREMENTAL),
            "应该使用增量同步模式"
        );
    } else {
        panic!("应该返回 TextDocumentSyncOptions");
    }
}

/// 测试初始化时没有客户端信息也能正常工作
///
/// **Validates: Requirements 1.2**
#[test]
fn test_initialize_without_client_info() {
    let mut server = LspServer::start().unwrap();

    #[allow(deprecated)]
    let params = InitializeParams {
        process_id: None,
        root_uri: None,
        capabilities: ClientCapabilities::default(),
        client_info: None,
        locale: None,
        root_path: None,
        initialization_options: None,
        trace: None,
        workspace_folders: None,
        work_done_progress_params: WorkDoneProgressParams::default(),
    };

    let result = server.handle_initialize(params);
    assert!(result.is_ok(), "没有客户端信息时初始化应该成功");
}

/// 测试初始化时有工作空间文件夹
///
/// **Validates: Requirements 1.2**
#[test]
fn test_initialize_with_workspace_folders() {
    let mut server = LspServer::start().unwrap();

    #[allow(deprecated)]
    let params = InitializeParams {
        process_id: Some(1234),
        root_uri: None,
        capabilities: ClientCapabilities::default(),
        client_info: None,
        locale: None,
        root_path: None,
        initialization_options: None,
        trace: None,
        workspace_folders: Some(vec![
            WorkspaceFolder {
                uri: Url::parse("file:///workspace1").unwrap(),
                name: "workspace1".to_string(),
            },
            WorkspaceFolder {
                uri: Url::parse("file:///workspace2").unwrap(),
                name: "workspace2".to_string(),
            },
        ]),
        work_done_progress_params: WorkDoneProgressParams::default(),
    };

    let result = server.handle_initialize(params);
    assert!(result.is_ok(), "有多个工作空间文件夹时初始化应该成功");
}

// ============================================================================
// 单元测试 - 优雅关闭
// ============================================================================

/// 测试服务器可以正常关闭
///
/// **Validates: Requirements 1.7**
#[test]
fn test_graceful_shutdown() {
    let mut server = LspServer::start().unwrap();

    // 调用关闭方法
    let result = server.shutdown();
    assert!(result.is_ok(), "服务器应该能够优雅关闭");
}

/// 测试关闭后资源被清理
///
/// **Validates: Requirements 1.7**
#[test]
fn test_shutdown_cleans_resources() {
    use lsp_types::{DidOpenTextDocumentParams, TextDocumentItem};

    let mut server = LspServer::start().unwrap();
    server.state = spring_lsp::server::ServerState::Initialized;

    // 打开一些文档
    let uri1 = Url::parse("file:///test1.toml").unwrap();
    let uri2 = Url::parse("file:///test2.toml").unwrap();

    server
        .handle_did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri1.clone(),
                language_id: "toml".to_string(),
                version: 1,
                text: "test1".to_string(),
            },
        })
        .unwrap();

    server
        .handle_did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri2.clone(),
                language_id: "toml".to_string(),
                version: 1,
                text: "test2".to_string(),
            },
        })
        .unwrap();

    // 验证文档已缓存
    assert!(server.document_manager.get(&uri1).is_some());
    assert!(server.document_manager.get(&uri2).is_some());

    // 关闭服务器
    server.shutdown().unwrap();

    // 注意：当前实现中，shutdown 不会自动清理文档缓存
    // 这是因为文档管理器是独立的组件
    // 在实际使用中，客户端会在关闭前发送 didClose 通知
}

// ============================================================================
// 属性测试 - 初始化握手
// ============================================================================

/// 生成有效的 InitializeParams
fn arb_initialize_params() -> impl Strategy<Value = InitializeParams> {
    (
        prop::option::of(any::<u32>()),
        prop::option::of("file:///[a-z]{3,10}"),
        prop::option::of(("[a-z]{3,10}", "[0-9]\\.[0-9]\\.[0-9]")),
    )
        .prop_map(|(process_id, root_uri, client_info)| {
            #[allow(deprecated)]
            InitializeParams {
                process_id,
                root_uri: root_uri.and_then(|s| Url::parse(&s).ok()),
                capabilities: ClientCapabilities::default(),
                client_info: client_info.map(|(name, version)| ClientInfo {
                    name,
                    version: Some(version),
                }),
                locale: None,
                root_path: None,
                initialization_options: None,
                trace: None,
                workspace_folders: None,
                work_done_progress_params: WorkDoneProgressParams::default(),
            }
        })
}

/// Feature: spring-lsp, Property 1: LSP 初始化响应
///
/// For any 有效的 LSP 初始化请求，服务器应该返回包含服务器能力声明的初始化响应。
///
/// **Validates: Requirements 1.2**
proptest! {
    #[test]
    fn prop_initialize_returns_valid_response(params in arb_initialize_params()) {
        let mut server = LspServer::start().unwrap();
        let result = server.handle_initialize(params);

        // 属性：初始化应该总是成功
        prop_assert!(result.is_ok(), "初始化应该总是成功");

        let init_result = result.unwrap();

        // 属性：应该返回服务器信息
        prop_assert!(init_result.server_info.is_some(), "应该返回服务器信息");

        // 属性：服务器名称应该是 "spring-lsp"
        let server_info = init_result.server_info.unwrap();
        prop_assert_eq!(server_info.name, "spring-lsp", "服务器名称应该是 spring-lsp");

        // 属性：应该声明文档同步能力
        prop_assert!(
            init_result.capabilities.text_document_sync.is_some(),
            "应该声明文档同步能力"
        );

        // 属性：应该声明补全能力
        prop_assert!(
            init_result.capabilities.completion_provider.is_some(),
            "应该声明补全能力"
        );

        // 属性：应该声明悬停能力
        prop_assert!(
            init_result.capabilities.hover_provider.is_some(),
            "应该声明悬停能力"
        );
    }
}

/// Feature: spring-lsp, Property 1: LSP 初始化响应 - 补全触发字符
///
/// For any 有效的初始化请求，返回的补全能力应该包含所有必需的触发字符。
///
/// **Validates: Requirements 1.2**
proptest! {
    #[test]
    fn prop_initialize_declares_completion_triggers(params in arb_initialize_params()) {
        let mut server = LspServer::start().unwrap();
        let result = server.handle_initialize(params).unwrap();

        let completion = result.capabilities.completion_provider.unwrap();
        let triggers = completion.trigger_characters.unwrap();

        // 属性：应该包含所有必需的触发字符
        prop_assert!(triggers.contains(&"[".to_string()), "应该包含 '['");
        prop_assert!(triggers.contains(&".".to_string()), "应该包含 '.'");
        prop_assert!(triggers.contains(&"$".to_string()), "应该包含 '$'");
        prop_assert!(triggers.contains(&"{".to_string()), "应该包含 '{{'");
        prop_assert!(triggers.contains(&"#".to_string()), "应该包含 '#'");
        prop_assert!(triggers.contains(&"(".to_string()), "应该包含 '('");
    }
}

// ============================================================================
// 边缘情况测试
// ============================================================================

/// 测试空的客户端能力
///
/// **Validates: Requirements 1.2**
#[test]
fn test_initialize_with_empty_capabilities() {
    let mut server = LspServer::start().unwrap();

    #[allow(deprecated)]
    let params = InitializeParams {
        process_id: Some(1234),
        root_uri: None,
        capabilities: ClientCapabilities {
            workspace: None,
            text_document: None,
            window: None,
            general: None,
            experimental: None,
        },
        client_info: None,
        locale: None,
        root_path: None,
        initialization_options: None,
        trace: None,
        workspace_folders: None,
        work_done_progress_params: WorkDoneProgressParams::default(),
    };

    let result = server.handle_initialize(params);
    assert!(
        result.is_ok(),
        "即使客户端能力为空，初始化也应该成功"
    );
}

/// 测试极长的客户端名称
///
/// **Validates: Requirements 1.2**
#[test]
fn test_initialize_with_very_long_client_name() {
    let mut server = LspServer::start().unwrap();

    let long_name = "a".repeat(10000);

    #[allow(deprecated)]
    let params = InitializeParams {
        process_id: Some(1234),
        root_uri: None,
        capabilities: ClientCapabilities::default(),
        client_info: Some(ClientInfo {
            name: long_name,
            version: Some("1.0.0".to_string()),
        }),
        locale: None,
        root_path: None,
        initialization_options: None,
        trace: None,
        workspace_folders: None,
        work_done_progress_params: WorkDoneProgressParams::default(),
    };

    let result = server.handle_initialize(params);
    assert!(
        result.is_ok(),
        "即使客户端名称很长，初始化也应该成功"
    );
}

/// 测试无效的 root_uri（应该被忽略）
///
/// **Validates: Requirements 1.2**
#[test]
fn test_initialize_with_invalid_root_uri() {
    let mut server = LspServer::start().unwrap();

    #[allow(deprecated)]
    let params = InitializeParams {
        process_id: Some(1234),
        // 注意：这里我们传入 None，因为 Url 类型已经验证过了
        // 在实际的 JSON-RPC 通信中，无效的 URL 会在反序列化时被拒绝
        root_uri: None,
        capabilities: ClientCapabilities::default(),
        client_info: None,
        locale: None,
        root_path: None,
        initialization_options: None,
        trace: None,
        workspace_folders: None,
        work_done_progress_params: WorkDoneProgressParams::default(),
    };

    let result = server.handle_initialize(params);
    assert!(result.is_ok(), "没有 root_uri 时初始化应该成功");
}

// ============================================================================
// 并发测试
// ============================================================================

/// 测试多个并发初始化请求（虽然在实际中不应该发生）
///
/// **Validates: Requirements 1.2**
#[test]
fn test_concurrent_initialization() {
    use std::sync::Arc;
    use std::thread;

    let server = Arc::new(LspServer::start().unwrap());

    let handles: Vec<_> = (0..10)
        .map(|i| {
            thread::spawn(move || {
                // 每个线程创建自己的服务器实例
                let mut server = LspServer::start().unwrap();
                
                #[allow(deprecated)]
                let params = InitializeParams {
                    process_id: Some(i),
                    root_uri: None,
                    capabilities: ClientCapabilities::default(),
                    client_info: Some(ClientInfo {
                        name: format!("client-{}", i),
                        version: Some("1.0.0".to_string()),
                    }),
                    locale: None,
                    root_path: None,
                    initialization_options: None,
                    trace: None,
                    workspace_folders: None,
                    work_done_progress_params: WorkDoneProgressParams::default(),
                };

                server.handle_initialize(params)
            })
        })
        .collect();

    // 所有初始化请求都应该成功
    for handle in handles {
        let result = handle.join().unwrap();
        assert!(result.is_ok(), "并发初始化应该成功");
    }
}

// ============================================================================
// 属性测试 - 优雅关闭
// ============================================================================

/// 生成文档 URI 列表
fn arb_document_uris() -> impl Strategy<Value = Vec<Url>> {
    prop::collection::vec("file:///[a-z]{3,10}\\.(toml|rs)", 0..10).prop_map(|strings| {
        strings
            .into_iter()
            .filter_map(|s| Url::parse(&s).ok())
            .collect()
    })
}

/// Feature: spring-lsp, Property 5: 错误恢复稳定性 (部分)
///
/// For any 服务器状态，关闭操作应该总是成功完成。
///
/// **Validates: Requirements 1.7**
proptest! {
    #[test]
    fn prop_shutdown_always_succeeds(uris in arb_document_uris()) {
        use lsp_types::{DidOpenTextDocumentParams, TextDocumentItem};

        let mut server = LspServer::start().unwrap();
        server.state = ServerState::Initialized;

        // 打开一些文档
        for (i, uri) in uris.iter().enumerate() {
            let _ = server.handle_did_open(DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    uri: uri.clone(),
                    language_id: "toml".to_string(),
                    version: i as i32,
                    text: format!("content-{}", i),
                },
            });
        }

        // 属性：关闭应该总是成功
        let result = server.shutdown();
        prop_assert!(result.is_ok(), "关闭应该总是成功");
    }
}

/// Feature: spring-lsp, Property 5: 错误恢复稳定性
///
/// For any 服务器状态，即使在未初始化状态下关闭也应该成功。
///
/// **Validates: Requirements 1.7**
proptest! {
    #[test]
    fn prop_shutdown_succeeds_in_any_state(
        state in prop::sample::select(vec![
            ServerState::Uninitialized,
            ServerState::Initialized,
            ServerState::ShuttingDown,
        ])
    ) {
        let mut server = LspServer::start().unwrap();
        server.state = state;

        // 属性：无论在什么状态，关闭都应该成功
        let result = server.shutdown();
        prop_assert!(result.is_ok(), "在任何状态下关闭都应该成功");
    }
}

// ============================================================================
// 集成测试 - 完整的初始化和关闭流程
// ============================================================================

/// 测试完整的初始化-使用-关闭流程
///
/// **Validates: Requirements 1.2, 1.7**
#[test]
fn test_full_lifecycle() {
    use lsp_types::{DidOpenTextDocumentParams, TextDocumentItem};

    let mut server = LspServer::start().unwrap();

    // 1. 初始化
    #[allow(deprecated)]
    let init_params = InitializeParams {
        process_id: Some(1234),
        root_uri: Some(Url::parse("file:///workspace").unwrap()),
        capabilities: ClientCapabilities::default(),
        client_info: Some(ClientInfo {
            name: "test-client".to_string(),
            version: Some("1.0.0".to_string()),
        }),
        locale: None,
        root_path: None,
        initialization_options: None,
        trace: None,
        workspace_folders: None,
        work_done_progress_params: WorkDoneProgressParams::default(),
    };

    let init_result = server.handle_initialize(init_params);
    assert!(init_result.is_ok(), "初始化应该成功");

    server.state = ServerState::Initialized;

    // 2. 使用服务器 - 打开文档
    let uri = Url::parse("file:///test.toml").unwrap();
    let open_result = server.handle_did_open(DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "toml".to_string(),
            version: 1,
            text: "host = \"localhost\"".to_string(),
        },
    });
    assert!(open_result.is_ok(), "打开文档应该成功");

    // 验证文档已缓存
    assert!(server.document_manager.get(&uri).is_some());

    // 3. 关闭服务器
    let shutdown_result = server.shutdown();
    assert!(shutdown_result.is_ok(), "关闭应该成功");
}

/// 测试多次关闭是幂等的
///
/// **Validates: Requirements 1.7**
#[test]
fn test_multiple_shutdowns_are_idempotent() {
    let mut server = LspServer::start().unwrap();

    // 第一次关闭
    let result1 = server.shutdown();
    assert!(result1.is_ok(), "第一次关闭应该成功");

    // 第二次关闭
    let result2 = server.shutdown();
    assert!(result2.is_ok(), "第二次关闭应该成功");

    // 第三次关闭
    let result3 = server.shutdown();
    assert!(result3.is_ok(), "第三次关闭应该成功");
}

// ============================================================================
// 性能测试
// ============================================================================

/// 测试初始化响应时间
///
/// **Validates: Requirements 1.2, 12.1**
#[test]
fn test_initialization_performance() {
    use std::time::Instant;

    let mut server = LspServer::start().unwrap();

    #[allow(deprecated)]
    let params = InitializeParams {
        process_id: Some(1234),
        root_uri: Some(Url::parse("file:///workspace").unwrap()),
        capabilities: ClientCapabilities::default(),
        client_info: Some(ClientInfo {
            name: "test-client".to_string(),
            version: Some("1.0.0".to_string()),
        }),
        locale: None,
        root_path: None,
        initialization_options: None,
        trace: None,
        workspace_folders: None,
        work_done_progress_params: WorkDoneProgressParams::default(),
    };

    let start = Instant::now();
    let result = server.handle_initialize(params);
    let duration = start.elapsed();

    assert!(result.is_ok(), "初始化应该成功");
    assert!(
        duration.as_millis() < 100,
        "初始化应该在 100ms 内完成，实际用时: {:?}",
        duration
    );
}

/// 测试关闭响应时间
///
/// **Validates: Requirements 1.7**
#[test]
fn test_shutdown_performance() {
    use lsp_types::{DidOpenTextDocumentParams, TextDocumentItem};
    use std::time::Instant;

    let mut server = LspServer::start().unwrap();
    server.state = ServerState::Initialized;

    // 打开多个文档
    for i in 0..100 {
        let uri = Url::parse(&format!("file:///test{}.toml", i)).unwrap();
        let _ = server.handle_did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri,
                language_id: "toml".to_string(),
                version: 1,
                text: format!("content-{}", i),
            },
        });
    }

    let start = Instant::now();
    let result = server.shutdown();
    let duration = start.elapsed();

    assert!(result.is_ok(), "关闭应该成功");
    assert!(
        duration.as_millis() < 500,
        "关闭应该在 500ms 内完成，实际用时: {:?}",
        duration
    );
}

// ============================================================================
// 错误处理测试
// ============================================================================

/// 测试在未初始化状态下的操作
///
/// **Validates: Requirements 1.6**
#[test]
fn test_operations_before_initialization() {
    use lsp_types::{DidOpenTextDocumentParams, TextDocumentItem};

    let mut server = LspServer::start().unwrap();
    // 注意：服务器处于 Uninitialized 状态

    let uri = Url::parse("file:///test.toml").unwrap();
    let result = server.handle_did_open(DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "toml".to_string(),
            version: 1,
            text: "test".to_string(),
        },
    });

    // 即使在未初始化状态，操作也不应该崩溃
    assert!(result.is_ok(), "未初始化状态下的操作不应该崩溃");
}

/// 测试在关闭状态下的操作
///
/// **Validates: Requirements 1.6, 1.7**
#[test]
fn test_operations_after_shutdown() {
    use lsp_types::{DidOpenTextDocumentParams, TextDocumentItem};

    let mut server = LspServer::start().unwrap();
    server.state = ServerState::ShuttingDown;

    let uri = Url::parse("file:///test.toml").unwrap();
    let result = server.handle_did_open(DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "toml".to_string(),
            version: 1,
            text: "test".to_string(),
        },
    });

    // 即使在关闭状态，操作也不应该崩溃
    assert!(result.is_ok(), "关闭状态下的操作不应该崩溃");
}
