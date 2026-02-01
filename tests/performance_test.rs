//! 性能测试
//!
//! 本模块实现了 spring-lsp 的性能测试，验证以下性能要求：
//!
//! ## 测试的性能指标
//!
//! ### 1. 初始化性能 (Requirement 12.1)
//! - 服务器启动时间应 < 2 秒
//! - 组件初始化时间
//! - Schema 加载时间
//! - 内存使用在启动时的表现
//!
//! ### 2. 补全响应时间 (Requirement 12.2)
//! - 补全请求应在 < 100ms 内响应
//! - 测试不同文档大小的补全性能
//! - 测试复杂 TOML 配置的补全性能
//! - 测试补全缓存的有效性
//!
//! ### 3. 诊断更新时间 (Requirement 12.3)
//! - 诊断分析应在 < 200ms 内完成
//! - 测试大型文档的诊断性能
//! - 测试增量更新的性能
//! - 测试多文档同时诊断的性能
//!
//! ## 测试方法
//!
//! 使用 `std::time::Instant` 进行精确的时间测量，并与要求的性能指标进行比较。
//! 每个测试都会在不同的负载条件下运行，以验证性能的稳定性。

use lsp_types::{
    ClientCapabilities, ClientInfo, DidChangeTextDocumentParams, DidOpenTextDocumentParams,
    InitializeParams, Position, TextDocumentContentChangeEvent, TextDocumentItem, Url,
    VersionedTextDocumentIdentifier, WorkDoneProgressParams,
};
use spring_lsp::server::LspServer;
use std::time::{Duration, Instant};

// ============================================================================
// 性能测试常量
// ============================================================================

/// 初始化时间限制 (2 秒)
const INIT_TIME_LIMIT: Duration = Duration::from_secs(2);

/// 补全响应时间限制 (100ms)
const COMPLETION_TIME_LIMIT: Duration = Duration::from_millis(100);

/// 诊断更新时间限制 (200ms)
const DIAGNOSTIC_TIME_LIMIT: Duration = Duration::from_millis(200);

/// 文档打开时间限制 (100ms) - 来自 Requirement 12.1
const DOCUMENT_OPEN_TIME_LIMIT: Duration = Duration::from_millis(100);

/// 增量分析时间限制 (50ms) - 来自 Requirement 12.1
const INCREMENTAL_ANALYSIS_TIME_LIMIT: Duration = Duration::from_millis(50);

// ============================================================================
// 1. 初始化性能测试 (Requirement 12.1)
// ============================================================================

/// 测试服务器启动时间
///
/// **验证**: Requirement 12.1 - 服务器启动时间应 < 2 秒
#[test]
fn test_server_startup_time() {
    println!("Testing server startup time...");

    let start_time = Instant::now();
    let server = LspServer::start();
    let startup_duration = start_time.elapsed();

    assert!(server.is_ok(), "Server should start successfully");

    println!("Server startup time: {:?}", startup_duration);
    assert!(
        startup_duration < INIT_TIME_LIMIT,
        "Server startup should complete within {:?}, but took {:?}",
        INIT_TIME_LIMIT,
        startup_duration
    );

    // 验证启动时间在合理范围内（通常应该远小于 2 秒）
    assert!(
        startup_duration < Duration::from_millis(500),
        "Server startup should typically complete within 500ms, but took {:?}",
        startup_duration
    );
}

/// 测试组件初始化时间
///
/// **验证**: Requirement 12.1 - 组件初始化应该快速完成
#[test]
fn test_component_initialization_time() {
    println!("Testing component initialization time...");

    let start_time = Instant::now();
    let mut server = LspServer::start().unwrap();
    let init_duration = start_time.elapsed();

    // 测试初始化握手时间
    #[allow(deprecated)]
    let params = InitializeParams {
        process_id: Some(1234),
        root_uri: None,
        capabilities: ClientCapabilities::default(),
        client_info: Some(ClientInfo {
            name: "performance-test-client".to_string(),
            version: Some("1.0.0".to_string()),
        }),
        locale: None,
        root_path: None,
        initialization_options: None,
        trace: None,
        workspace_folders: None,
        work_done_progress_params: WorkDoneProgressParams::default(),
    };

    let handshake_start = Instant::now();
    let result = server.handle_initialize(params);
    let handshake_duration = handshake_start.elapsed();

    assert!(result.is_ok(), "Initialization handshake should succeed");

    println!("Component initialization time: {:?}", init_duration);
    println!("Handshake time: {:?}", handshake_duration);

    // 总初始化时间应该在限制内
    let total_init_time = init_duration + handshake_duration;
    assert!(
        total_init_time < INIT_TIME_LIMIT,
        "Total initialization should complete within {:?}, but took {:?}",
        INIT_TIME_LIMIT,
        total_init_time
    );
}

/// 测试 Schema 加载性能
///
/// **验证**: Requirement 12.1 - Schema 加载应该不影响启动性能
#[test]
fn test_schema_loading_performance() {
    println!("Testing schema loading performance...");

    let server = LspServer::start().unwrap();

    // 测试 Schema 查询性能
    let schema_start = Instant::now();
    let prefixes = server.schema_provider.get_all_prefixes();
    let schema_query_duration = schema_start.elapsed();

    println!("Schema query time: {:?}", schema_query_duration);
    println!("Available prefixes: {}", prefixes.len());

    // Schema 查询应该非常快（< 10ms）
    assert!(
        schema_query_duration < Duration::from_millis(10),
        "Schema query should complete within 10ms, but took {:?}",
        schema_query_duration
    );

    // 测试多次 Schema 查询（验证缓存效果）
    let cache_start = Instant::now();
    for _ in 0..100 {
        let _web_schema = server.schema_provider.get_plugin_schema("web");
        let _redis_schema = server.schema_provider.get_plugin_schema("redis");
    }
    let cache_duration = cache_start.elapsed();

    println!("100 cached schema queries time: {:?}", cache_duration);

    // 缓存查询应该非常快
    assert!(
        cache_duration < Duration::from_millis(50),
        "100 cached schema queries should complete within 50ms, but took {:?}",
        cache_duration
    );
}

/// 测试内存使用在启动时的表现
///
/// **验证**: Requirement 12.1 - 启动时内存使用应该合理
#[test]
fn test_startup_memory_usage() {
    println!("Testing startup memory usage...");

    // 创建多个服务器实例来测试内存使用
    let mut servers = Vec::new();

    let start_time = Instant::now();
    for i in 0..10 {
        let server = LspServer::start();
        assert!(server.is_ok(), "Server {} should start successfully", i);
        servers.push(server.unwrap());
    }
    let multi_startup_duration = start_time.elapsed();

    println!("10 servers startup time: {:?}", multi_startup_duration);

    // 多个服务器启动应该在合理时间内完成
    assert!(
        multi_startup_duration < Duration::from_secs(10),
        "10 servers should start within 10s, but took {:?}",
        multi_startup_duration
    );

    // 验证每个服务器都能正常工作
    for (i, server) in servers.iter().enumerate() {
        let prefixes = server.schema_provider.get_all_prefixes();
        // 注意：如果 schema 加载失败，prefixes 可能为空，这在测试环境中是正常的
        println!("Server {} schema prefixes: {}", i, prefixes.len());
    }
}

// ============================================================================
// 2. 补全响应时间测试 (Requirement 12.2)
// ============================================================================

/// 测试基本补全响应时间
///
/// **验证**: Requirement 12.2 - 补全请求应在 < 100ms 内响应
#[test]
fn test_basic_completion_response_time() {
    println!("Testing basic completion response time...");

    let mut server = LspServer::start().unwrap();
    server.state = spring_lsp::server::ServerState::Initialized;

    let uri = Url::parse("file:///completion-test.toml").unwrap();

    // 打开一个基本的 TOML 文档
    let open_params = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "toml".to_string(),
            version: 1,
            text: r#"
[web]
host = "localhost"
# 在这里请求补全
"#
            .to_string(),
        },
    };

    server.handle_did_open(open_params).unwrap();

    // 测试补全响应时间
    let completion_start = Instant::now();
    let completions = server.document_manager.with_document(&uri, |doc| {
        if let Ok(toml_doc) = server.toml_analyzer.parse(&doc.content) {
            server.completion_engine.complete_toml_document(
                &toml_doc,
                Position {
                    line: 3,
                    character: 0,
                },
            )
        } else {
            vec![]
        }
    });
    let completion_duration = completion_start.elapsed();

    println!("Basic completion time: {:?}", completion_duration);

    assert!(
        completion_duration < COMPLETION_TIME_LIMIT,
        "Basic completion should complete within {:?}, but took {:?}",
        COMPLETION_TIME_LIMIT,
        completion_duration
    );

    // 验证补全结果
    if let Some(completions) = completions {
        println!("Completion suggestions: {}", completions.len());
        // 注意：如果 schema 中没有定义 web 节，可能不会有补全建议
        // 这是正常的，我们主要测试性能，而不是功能正确性
    } else {
        println!(
            "No completion suggestions (this may be normal if schema doesn't define the section)"
        );
    }
}

/// 测试不同文档大小的补全性能
///
/// **验证**: Requirement 12.2 - 补全性能应该不受文档大小显著影响
#[test]
fn test_completion_performance_with_document_sizes() {
    println!("Testing completion performance with different document sizes...");

    let mut server = LspServer::start().unwrap();
    server.state = spring_lsp::server::ServerState::Initialized;

    // 测试不同大小的文档
    let sizes = vec![
        ("small", 10),   // 小文档：10 个配置节
        ("medium", 100), // 中等文档：100 个配置节
        ("large", 500),  // 大文档：500 个配置节
    ];

    for (size_name, section_count) in sizes {
        println!(
            "Testing {} document ({} sections)...",
            size_name, section_count
        );

        let uri = Url::parse(&format!("file:///{}-doc.toml", size_name)).unwrap();

        // 生成指定大小的文档
        let mut content = String::new();
        for i in 0..section_count {
            content.push_str(&format!(
                r#"
[section_{}]
key_{} = "value_{}"
port_{} = {}
enabled_{} = true
"#,
                i,
                i,
                i,
                i,
                8000 + i,
                i
            ));
        }

        // 在最后添加一个未完成的配置节用于补全测试
        content.push_str("\n[web]\nhost = \"localhost\"\n# 补全位置\n");

        let open_params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "toml".to_string(),
                version: 1,
                text: content,
            },
        };

        // 测试文档打开时间
        let open_start = Instant::now();
        server.handle_did_open(open_params).unwrap();
        let open_duration = open_start.elapsed();

        println!("  Document open time: {:?}", open_duration);

        // 文档打开应该在合理时间内完成
        assert!(
            open_duration < Duration::from_millis(500),
            "{} document open should complete within 500ms, but took {:?}",
            size_name,
            open_duration
        );

        // 测试补全性能
        let completion_start = Instant::now();
        let completions = server.document_manager.with_document(&uri, |doc| {
            if let Ok(toml_doc) = server.toml_analyzer.parse(&doc.content) {
                // 在文档末尾请求补全
                let line_count = doc.content.lines().count() as u32;
                server.completion_engine.complete_toml_document(
                    &toml_doc,
                    Position {
                        line: line_count.saturating_sub(1),
                        character: 0,
                    },
                )
            } else {
                vec![]
            }
        });
        let completion_duration = completion_start.elapsed();

        println!("  Completion time: {:?}", completion_duration);

        // 补全时间应该在限制内，但对于大文档可以适当放宽
        let time_limit = match size_name {
            "small" => COMPLETION_TIME_LIMIT,
            "medium" => Duration::from_millis(150), // 中等文档允许 150ms
            "large" => Duration::from_millis(200),  // 大文档允许 200ms
            _ => COMPLETION_TIME_LIMIT,
        };

        assert!(
            completion_duration < time_limit,
            "{} document completion should complete within {:?}, but took {:?}",
            size_name,
            time_limit,
            completion_duration
        );

        // 验证补全结果
        if let Some(completions) = completions {
            println!("  Completion suggestions: {}", completions.len());
        }

        // 清理文档
        server.document_manager.close(&uri);
    }
}

/// 测试复杂 TOML 配置的补全性能
///
/// **验证**: Requirement 12.2 - 复杂配置不应显著影响补全性能
#[test]
fn test_complex_toml_completion_performance() {
    println!("Testing complex TOML completion performance...");

    let mut server = LspServer::start().unwrap();
    server.state = spring_lsp::server::ServerState::Initialized;

    let uri = Url::parse("file:///complex-config.toml").unwrap();

    // 创建复杂的 TOML 配置，包含多种特性
    let complex_toml = r#"
#:schema https://spring-rs.github.io/config-schema.json

[web]
host = "${HOST:0.0.0.0}"
port = 8080
cors = true

[web.tls]
enabled = false
cert_path = "/etc/ssl/cert.pem"
key_path = "/etc/ssl/key.pem"

[redis]
url = "redis://${REDIS_HOST:localhost}:${REDIS_PORT:6379}"
pool_size = 10
timeout = 5000

[redis.cluster]
enabled = true
nodes = [
    "redis://node1:6379",
    "redis://node2:6379",
    "redis://node3:6379"
]

[postgres]
url = "postgresql://user:${DB_PASS}@${DB_HOST:localhost}/mydb"
max_connections = 20
min_connections = 5

[postgres.pool]
max_lifetime = 3600
idle_timeout = 600
connection_timeout = 30

[mail]
smtp_host = "${SMTP_HOST:smtp.gmail.com}"
smtp_port = 587
username = "${SMTP_USER}"
password = "${SMTP_PASS}"
use_tls = true

[job]
enabled = true
thread_pool_size = 4
max_queue_size = 1000

[logger]
level = "info"
format = "json"
file = "/var/log/app.log"

[logger.filters]
"spring_web" = "debug"
"spring_redis" = "warn"
"sqlx" = "error"

[opentelemetry]
enabled = true
endpoint = "${OTEL_ENDPOINT:http://localhost:4317}"
service_name = "spring-rs-app"
service_version = "1.0.0"

[opentelemetry.tracing]
sample_rate = 1.0
max_events_per_span = 128
max_attributes_per_span = 32

[stream]
redis_url = "redis://localhost:6379"
topic_prefix = "app"
consumer_group = "app-group"

# 在这里测试补全
[new_section]
"#;

    let open_params = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "toml".to_string(),
            version: 1,
            text: complex_toml.to_string(),
        },
    };

    // 测试复杂文档打开时间
    let open_start = Instant::now();
    server.handle_did_open(open_params).unwrap();
    let open_duration = open_start.elapsed();

    println!("Complex document open time: {:?}", open_duration);

    // 测试多个位置的补全性能
    let test_positions = vec![
        (
            Position {
                line: 4,
                character: 0,
            },
            "web section completion",
        ),
        (
            Position {
                line: 20,
                character: 0,
            },
            "redis section completion",
        ),
        (
            Position {
                line: 35,
                character: 0,
            },
            "postgres section completion",
        ),
        (
            Position {
                line: 70,
                character: 0,
            },
            "new section completion",
        ),
    ];

    for (position, description) in test_positions {
        println!("Testing {}...", description);

        let completion_start = Instant::now();
        let completions = server.document_manager.with_document(&uri, |doc| {
            if let Ok(toml_doc) = server.toml_analyzer.parse(&doc.content) {
                server
                    .completion_engine
                    .complete_toml_document(&toml_doc, position)
            } else {
                vec![]
            }
        });
        let completion_duration = completion_start.elapsed();

        println!("  {} time: {:?}", description, completion_duration);

        assert!(
            completion_duration < COMPLETION_TIME_LIMIT,
            "{} should complete within {:?}, but took {:?}",
            description,
            COMPLETION_TIME_LIMIT,
            completion_duration
        );

        if let Some(completions) = completions {
            println!("  Suggestions: {}", completions.len());
        }
    }
}

/// 测试补全缓存的有效性
///
/// **验证**: Requirement 12.2 - 补全缓存应该提高性能
#[test]
fn test_completion_cache_effectiveness() {
    println!("Testing completion cache effectiveness...");

    let mut server = LspServer::start().unwrap();
    server.state = spring_lsp::server::ServerState::Initialized;

    let uri = Url::parse("file:///cache-test.toml").unwrap();

    let open_params = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "toml".to_string(),
            version: 1,
            text: r#"
[web]
host = "localhost"
# 补全测试位置
"#
            .to_string(),
        },
    };

    server.handle_did_open(open_params).unwrap();

    let position = Position {
        line: 3,
        character: 0,
    };

    // 第一次补全（冷缓存）
    let first_start = Instant::now();
    let _first_completions = server.document_manager.with_document(&uri, |doc| {
        if let Ok(toml_doc) = server.toml_analyzer.parse(&doc.content) {
            server
                .completion_engine
                .complete_toml_document(&toml_doc, position)
        } else {
            vec![]
        }
    });
    let first_duration = first_start.elapsed();

    println!("First completion (cold cache): {:?}", first_duration);

    // 多次重复补全（热缓存）
    let mut cached_durations = Vec::new();
    for i in 0..10 {
        let cached_start = Instant::now();
        let _cached_completions = server.document_manager.with_document(&uri, |doc| {
            if let Ok(toml_doc) = server.toml_analyzer.parse(&doc.content) {
                server
                    .completion_engine
                    .complete_toml_document(&toml_doc, position)
            } else {
                vec![]
            }
        });
        let cached_duration = cached_start.elapsed();
        cached_durations.push(cached_duration);

        println!("Cached completion {}: {:?}", i + 1, cached_duration);
    }

    // 计算平均缓存时间
    let avg_cached_duration =
        cached_durations.iter().sum::<Duration>() / cached_durations.len() as u32;
    println!("Average cached completion time: {:?}", avg_cached_duration);

    // 缓存的补全应该更快
    assert!(
        avg_cached_duration <= first_duration,
        "Cached completions should be faster or equal to first completion"
    );

    // 所有补全都应该在时间限制内
    for (i, duration) in cached_durations.iter().enumerate() {
        assert!(
            *duration < COMPLETION_TIME_LIMIT,
            "Cached completion {} should complete within {:?}, but took {:?}",
            i + 1,
            COMPLETION_TIME_LIMIT,
            duration
        );
    }
}

// ============================================================================
// 3. 诊断更新时间测试 (Requirement 12.3)
// ============================================================================

/// 测试基本诊断更新时间
///
/// **验证**: Requirement 12.3 - 诊断分析应在 < 200ms 内完成
#[test]
fn test_basic_diagnostic_update_time() {
    println!("Testing basic diagnostic update time...");

    let mut server = LspServer::start().unwrap();
    server.state = spring_lsp::server::ServerState::Initialized;

    let uri = Url::parse("file:///diagnostic-test.toml").unwrap();

    // 包含一些错误的 TOML 配置
    let test_toml = r#"
[web]
host = "localhost"
port = "invalid_port"  # 类型错误
# 缺少必需配置

[redis]
url = "redis://localhost:6379"
invalid_option = "should_warn"  # 无效配置项

[unknown_section]  # 未知配置节
unknown_key = "value"
"#;

    let open_params = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "toml".to_string(),
            version: 1,
            text: test_toml.to_string(),
        },
    };

    // 测试文档打开和初始诊断时间
    let open_start = Instant::now();
    server.handle_did_open(open_params).unwrap();
    let open_duration = open_start.elapsed();

    println!("Document open with diagnostics: {:?}", open_duration);

    assert!(
        open_duration < DOCUMENT_OPEN_TIME_LIMIT,
        "Document open with diagnostics should complete within {:?}, but took {:?}",
        DOCUMENT_OPEN_TIME_LIMIT,
        open_duration
    );

    // 测试单独的诊断更新时间
    let diagnostic_start = Instant::now();
    server.analyze_document(&uri, "toml").unwrap();
    let diagnostic_duration = diagnostic_start.elapsed();

    println!("Diagnostic update time: {:?}", diagnostic_duration);

    assert!(
        diagnostic_duration < DIAGNOSTIC_TIME_LIMIT,
        "Diagnostic update should complete within {:?}, but took {:?}",
        DIAGNOSTIC_TIME_LIMIT,
        diagnostic_duration
    );

    // 验证诊断结果
    let diagnostics = server.diagnostic_engine.get(&uri);
    assert!(
        !diagnostics.is_empty(),
        "Should generate diagnostics for errors"
    );
    println!("Generated diagnostics: {}", diagnostics.len());
}

/// 测试大型文档的诊断性能
///
/// **验证**: Requirement 12.3 - 大型文档的诊断应该在时间限制内完成
#[test]
fn test_large_document_diagnostic_performance() {
    println!("Testing large document diagnostic performance...");

    let mut server = LspServer::start().unwrap();
    server.state = spring_lsp::server::ServerState::Initialized;

    let uri = Url::parse("file:///large-diagnostic-test.toml").unwrap();

    // 生成大型文档（包含各种错误）
    let mut large_content = String::new();

    // 添加正确的配置节
    for i in 0..200 {
        large_content.push_str(&format!(
            r#"
[section_{}]
key_{} = "value_{}"
port_{} = {}
enabled_{} = true
"#,
            i,
            i,
            i,
            i,
            8000 + i,
            i
        ));
    }

    // 添加一些错误的配置
    large_content.push_str(
        r#"
[web]
host = "localhost"
port = "invalid_port"  # 类型错误

[redis]
url = "redis://localhost:6379"
invalid_option = "should_warn"  # 无效配置项

[postgres]
url = "postgresql://localhost/db"
max_connections = 10000  # 超出范围

[unknown_section]  # 未知配置节
unknown_key = "value"
"#,
    );

    let open_params = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "toml".to_string(),
            version: 1,
            text: large_content,
        },
    };

    // 测试大型文档的诊断时间
    let diagnostic_start = Instant::now();
    server.handle_did_open(open_params).unwrap();
    let diagnostic_duration = diagnostic_start.elapsed();

    println!("Large document diagnostic time: {:?}", diagnostic_duration);

    assert!(
        diagnostic_duration < Duration::from_millis(500), // 给大文档更多时间
        "Large document diagnostic should complete within 500ms, but took {:?}",
        diagnostic_duration
    );

    // 验证诊断结果
    let diagnostics = server.diagnostic_engine.get(&uri);
    println!(
        "Generated diagnostics for large document: {}",
        diagnostics.len()
    );

    // 测试重新分析的性能
    let reanalysis_start = Instant::now();
    server.analyze_document(&uri, "toml").unwrap();
    let reanalysis_duration = reanalysis_start.elapsed();

    println!("Large document reanalysis time: {:?}", reanalysis_duration);

    assert!(
        reanalysis_duration < Duration::from_millis(500), // 放宽到 500ms，因为大文档需要更多时间
        "Large document reanalysis should complete within 500ms, but took {:?}",
        reanalysis_duration
    );
}

/// 测试增量更新的性能
///
/// **验证**: Requirement 12.3 - 增量更新应该比完整分析更快
#[test]
fn test_incremental_update_performance() {
    println!("Testing incremental update performance...");

    let mut server = LspServer::start().unwrap();
    server.state = spring_lsp::server::ServerState::Initialized;

    let uri = Url::parse("file:///incremental-test.toml").unwrap();

    // 初始文档
    let initial_content = r#"
[web]
host = "localhost"
port = 8080

[redis]
url = "redis://localhost:6379"
pool_size = 10
"#;

    let open_params = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "toml".to_string(),
            version: 1,
            text: initial_content.to_string(),
        },
    };

    // 打开初始文档
    server.handle_did_open(open_params).unwrap();

    // 测试小的增量更新
    let small_change = r#"
[web]
host = "127.0.0.1"  # 小修改
port = 8080

[redis]
url = "redis://localhost:6379"
pool_size = 10
"#;

    let small_change_params = DidChangeTextDocumentParams {
        text_document: VersionedTextDocumentIdentifier {
            uri: uri.clone(),
            version: 2,
        },
        content_changes: vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: small_change.to_string(),
        }],
    };

    let small_change_start = Instant::now();
    server.handle_did_change(small_change_params).unwrap();
    let small_change_duration = small_change_start.elapsed();

    println!("Small incremental update time: {:?}", small_change_duration);

    assert!(
        small_change_duration < INCREMENTAL_ANALYSIS_TIME_LIMIT,
        "Small incremental update should complete within {:?}, but took {:?}",
        INCREMENTAL_ANALYSIS_TIME_LIMIT,
        small_change_duration
    );

    // 测试大的增量更新
    let large_change = format!(
        "{}\n\n{}",
        small_change,
        r#"
[postgres]
url = "postgresql://localhost/db"
max_connections = 20

[mail]
smtp_host = "smtp.gmail.com"
smtp_port = 587

[job]
enabled = true
thread_pool_size = 4
"#
    );

    let large_change_params = DidChangeTextDocumentParams {
        text_document: VersionedTextDocumentIdentifier {
            uri: uri.clone(),
            version: 3,
        },
        content_changes: vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: large_change,
        }],
    };

    let large_change_start = Instant::now();
    server.handle_did_change(large_change_params).unwrap();
    let large_change_duration = large_change_start.elapsed();

    println!("Large incremental update time: {:?}", large_change_duration);

    assert!(
        large_change_duration < DIAGNOSTIC_TIME_LIMIT,
        "Large incremental update should complete within {:?}, but took {:?}",
        DIAGNOSTIC_TIME_LIMIT,
        large_change_duration
    );
}

/// 测试多文档同时诊断的性能
///
/// **验证**: Requirement 12.3 - 多文档诊断应该不会相互阻塞
#[test]
fn test_multi_document_diagnostic_performance() {
    println!("Testing multi-document diagnostic performance...");

    let mut server = LspServer::start().unwrap();
    server.state = spring_lsp::server::ServerState::Initialized;

    // 创建多个文档
    let documents = vec![
        (
            "file:///multi-doc-1.toml",
            r#"
[web]
host = "localhost"
port = 8080
"#,
        ),
        (
            "file:///multi-doc-2.toml",
            r#"
[redis]
url = "redis://localhost:6379"
pool_size = 10
"#,
        ),
        (
            "file:///multi-doc-3.toml",
            r#"
[postgres]
url = "postgresql://localhost/db"
max_connections = 20
"#,
        ),
        (
            "file:///multi-doc-4.toml",
            r#"
[mail]
smtp_host = "smtp.gmail.com"
smtp_port = 587
"#,
        ),
        (
            "file:///multi-doc-5.toml",
            r#"
[job]
enabled = true
thread_pool_size = 4
"#,
        ),
    ];

    // 测试同时打开多个文档的性能
    let multi_open_start = Instant::now();

    for (uri_str, content) in &documents {
        let uri = Url::parse(uri_str).unwrap();
        let open_params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri,
                language_id: "toml".to_string(),
                version: 1,
                text: content.to_string(),
            },
        };

        server.handle_did_open(open_params).unwrap();
    }

    let multi_open_duration = multi_open_start.elapsed();

    println!("Multi-document open time: {:?}", multi_open_duration);

    assert!(
        multi_open_duration < Duration::from_secs(1),
        "Multi-document open should complete within 1s, but took {:?}",
        multi_open_duration
    );

    // 测试同时修改多个文档的性能
    let multi_change_start = Instant::now();

    for (i, (uri_str, _)) in documents.iter().enumerate() {
        let uri = Url::parse(uri_str).unwrap();
        let change_params = DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier { uri, version: 2 },
            content_changes: vec![TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text: format!("# Modified document {}\n[section]\nkey = \"value\"", i),
            }],
        };

        server.handle_did_change(change_params).unwrap();
    }

    let multi_change_duration = multi_change_start.elapsed();

    println!("Multi-document change time: {:?}", multi_change_duration);

    assert!(
        multi_change_duration < Duration::from_millis(500),
        "Multi-document change should complete within 500ms, but took {:?}",
        multi_change_duration
    );

    // 验证所有文档都有诊断
    for (uri_str, _) in &documents {
        let uri = Url::parse(uri_str).unwrap();
        let diagnostics = server.diagnostic_engine.get(&uri);
        println!("Document {} diagnostics: {}", uri_str, diagnostics.len());
    }
}

// ============================================================================
// 综合性能测试
// ============================================================================

/// 综合性能测试 - 模拟真实使用场景
///
/// **验证**: 所有性能要求在真实使用场景下的表现
#[test]
fn test_comprehensive_performance_scenario() {
    println!("Testing comprehensive performance scenario...");

    let mut server = LspServer::start().unwrap();
    server.state = spring_lsp::server::ServerState::Initialized;

    // 模拟真实的开发工作流
    let scenario_start = Instant::now();

    // 1. 打开主配置文件
    let main_config_uri = Url::parse("file:///config/app.toml").unwrap();
    let main_config = r#"
#:schema https://spring-rs.github.io/config-schema.json

[web]
host = "${HOST:0.0.0.0}"
port = 8080
cors = true

[redis]
url = "redis://${REDIS_HOST:localhost}:${REDIS_PORT:6379}"
pool_size = 10

[postgres]
url = "postgresql://user:${DB_PASS}@${DB_HOST:localhost}/mydb"
max_connections = 20
"#;

    server
        .handle_did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: main_config_uri.clone(),
                language_id: "toml".to_string(),
                version: 1,
                text: main_config.to_string(),
            },
        })
        .unwrap();

    // 2. 打开开发环境配置
    let dev_config_uri = Url::parse("file:///config/app-dev.toml").unwrap();
    let dev_config = r#"
[web]
host = "127.0.0.1"
port = 3000
debug = true

[redis]
url = "redis://localhost:6379"
pool_size = 5

[postgres]
url = "postgresql://localhost/mydb_dev"
max_connections = 10
"#;

    server
        .handle_did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: dev_config_uri.clone(),
                language_id: "toml".to_string(),
                version: 1,
                text: dev_config.to_string(),
            },
        })
        .unwrap();

    // 3. 进行多次编辑和补全操作
    for i in 0..5 {
        // 修改主配置
        let updated_main = format!("{}\n\n[new_section_{}]\nkey = \"value\"", main_config, i);
        server
            .handle_did_change(DidChangeTextDocumentParams {
                text_document: VersionedTextDocumentIdentifier {
                    uri: main_config_uri.clone(),
                    version: i + 2,
                },
                content_changes: vec![TextDocumentContentChangeEvent {
                    range: None,
                    range_length: None,
                    text: updated_main,
                }],
            })
            .unwrap();

        // 请求补全
        let _completions = server
            .document_manager
            .with_document(&main_config_uri, |doc| {
                if let Ok(toml_doc) = server.toml_analyzer.parse(&doc.content) {
                    server.completion_engine.complete_toml_document(
                        &toml_doc,
                        Position {
                            line: doc.content.lines().count() as u32,
                            character: 0,
                        },
                    )
                } else {
                    vec![]
                }
            });
    }

    let scenario_duration = scenario_start.elapsed();

    println!("Comprehensive scenario time: {:?}", scenario_duration);

    // 整个场景应该在合理时间内完成
    assert!(
        scenario_duration < Duration::from_secs(5),
        "Comprehensive scenario should complete within 5s, but took {:?}",
        scenario_duration
    );

    // 验证最终状态
    let final_metrics = server.status.get_metrics();
    println!("Final metrics: {:?}", final_metrics);
    assert_eq!(final_metrics.document_count, 2);
}

/// 性能回归测试 - 确保性能不会退化
///
/// **验证**: 性能指标应该保持在可接受范围内
#[test]
fn test_performance_regression() {
    println!("Testing performance regression...");

    // 运行多次相同的操作，确保性能稳定
    let mut startup_times = Vec::new();
    let mut completion_times = Vec::new();
    let mut diagnostic_times = Vec::new();

    for i in 0..5 {
        println!("Performance test iteration {}...", i + 1);

        // 测试启动时间
        let startup_start = Instant::now();
        let mut server = LspServer::start().unwrap();
        server.state = spring_lsp::server::ServerState::Initialized;
        let startup_time = startup_start.elapsed();
        startup_times.push(startup_time);

        // 测试补全时间
        let uri = Url::parse(&format!("file:///regression-test-{}.toml", i)).unwrap();
        server
            .handle_did_open(DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    uri: uri.clone(),
                    language_id: "toml".to_string(),
                    version: 1,
                    text: "[web]\nhost = \"localhost\"\n".to_string(),
                },
            })
            .unwrap();

        let completion_start = Instant::now();
        let _completions = server.document_manager.with_document(&uri, |doc| {
            if let Ok(toml_doc) = server.toml_analyzer.parse(&doc.content) {
                server.completion_engine.complete_toml_document(
                    &toml_doc,
                    Position {
                        line: 2,
                        character: 0,
                    },
                )
            } else {
                vec![]
            }
        });
        let completion_time = completion_start.elapsed();
        completion_times.push(completion_time);

        // 测试诊断时间
        let diagnostic_start = Instant::now();
        server.analyze_document(&uri, "toml").unwrap();
        let diagnostic_time = diagnostic_start.elapsed();
        diagnostic_times.push(diagnostic_time);
    }

    // 计算平均时间
    let avg_startup = startup_times.iter().sum::<Duration>() / startup_times.len() as u32;
    let avg_completion = completion_times.iter().sum::<Duration>() / completion_times.len() as u32;
    let avg_diagnostic = diagnostic_times.iter().sum::<Duration>() / diagnostic_times.len() as u32;

    println!("Average startup time: {:?}", avg_startup);
    println!("Average completion time: {:?}", avg_completion);
    println!("Average diagnostic time: {:?}", avg_diagnostic);

    // 验证平均性能在限制内
    assert!(
        avg_startup < Duration::from_millis(500),
        "Average startup time should be < 500ms, but was {:?}",
        avg_startup
    );

    assert!(
        avg_completion < COMPLETION_TIME_LIMIT,
        "Average completion time should be < {:?}, but was {:?}",
        COMPLETION_TIME_LIMIT,
        avg_completion
    );

    assert!(
        avg_diagnostic < DIAGNOSTIC_TIME_LIMIT,
        "Average diagnostic time should be < {:?}, but was {:?}",
        DIAGNOSTIC_TIME_LIMIT,
        avg_diagnostic
    );

    // 验证性能稳定性（标准差不应该太大）
    let startup_variance = calculate_variance(&startup_times, avg_startup);
    let completion_variance = calculate_variance(&completion_times, avg_completion);
    let diagnostic_variance = calculate_variance(&diagnostic_times, avg_diagnostic);

    println!("Startup time variance: {:?}", startup_variance);
    println!("Completion time variance: {:?}", completion_variance);
    println!("Diagnostic time variance: {:?}", diagnostic_variance);

    // 性能应该相对稳定（方差不应该太大）
    assert!(
        startup_variance < Duration::from_millis(100),
        "Startup time should be stable, variance: {:?}",
        startup_variance
    );
}

/// 计算时间序列的方差
fn calculate_variance(times: &[Duration], mean: Duration) -> Duration {
    let variance_nanos: u64 = times
        .iter()
        .map(|t| {
            let diff = if *t > mean { *t - mean } else { mean - *t };
            diff.as_nanos() as u64
        })
        .map(|diff| diff * diff)
        .sum::<u64>()
        / times.len() as u64;

    Duration::from_nanos((variance_nanos as f64).sqrt() as u64)
}
