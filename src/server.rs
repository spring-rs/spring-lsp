//! LSP 服务器核心实现
//!
//! 本模块实现了 spring-lsp 的核心 LSP 服务器功能，包括：
//!
//! ## 服务器能力
//!
//! ### 文档同步 (Text Document Sync)
//! - 支持文档打开、修改、关闭通知
//! - 使用增量更新模式 (INCREMENTAL) 提高性能
//! - 自动缓存和管理文档内容
//!
//! ### 智能补全 (Completion)
//! - TOML 配置文件：配置节、配置项、枚举值补全
//! - Rust 代码：宏参数补全
//! - 环境变量：`${VAR:default}` 格式的环境变量补全
//! - 触发字符：`[`, `.`, `$`, `{`, `#`, `(`
//!
//! ### 悬停提示 (Hover)
//! - 配置项：显示类型、文档、默认值
//! - 宏：显示宏展开后的代码
//! - 路由：显示完整路径和 HTTP 方法
//! - 环境变量：显示当前值（如果可用）
//!
//! ### 定义跳转 (Go to Definition)
//! - 路由路径：跳转到处理器函数定义
//! - 组件注入：跳转到组件定义
//!
//! ### 文档符号 (Document Symbols)
//! - 显示文档中的所有路由
//! - 显示配置节和配置项
//!
//! ### 工作空间符号 (Workspace Symbols)
//! - 全局搜索路由
//! - 全局搜索组件
//!
//! ### 诊断 (Diagnostics)
//! - 配置验证：类型检查、必需项检查、废弃警告
//! - 路由验证：路径语法、参数类型、冲突检测
//! - 依赖注入验证：组件存在性、循环依赖检测
//!
//! ## LSP 协议版本
//!
//! 本实现遵循 LSP 3.17 规范。

use crate::document::DocumentManager;
use crate::{Error, Result};
use lsp_server::{Connection, Message, Notification, Request, RequestId, Response};
use lsp_types::{
    notification::{
        DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, Exit, Notification as _,
    },
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    InitializeParams, InitializeResult, ServerCapabilities, ServerInfo,
};
use std::sync::Arc;

/// 服务器状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerState {
    /// 未初始化
    Uninitialized,
    /// 已初始化
    Initialized,
    /// 正在关闭
    ShuttingDown,
}

/// LSP 服务器
pub struct LspServer {
    /// LSP 连接
    connection: Connection,
    /// 服务器状态
    pub state: ServerState,
    /// 文档管理器
    pub document_manager: Arc<DocumentManager>,
}

impl LspServer {
    /// 启动 LSP 服务器
    ///
    /// 这个方法创建服务器实例并初始化 LSP 连接
    pub fn start() -> Result<Self> {
        tracing::info!("Starting spring-lsp server");

        // 通过标准输入输出创建 LSP 连接
        let (connection, _io_threads) = Connection::stdio();

        Ok(Self {
            connection,
            state: ServerState::Uninitialized,
            document_manager: Arc::new(DocumentManager::new()),
        })
    }

    /// 运行服务器主循环
    ///
    /// 这个方法处理初始化握手，然后进入主事件循环处理来自客户端的消息
    pub fn run(&mut self) -> Result<()> {
        // 处理初始化握手
        self.initialize()?;

        // 主事件循环
        self.event_loop()?;

        // 优雅关闭
        self.shutdown()?;

        Ok(())
    }

    /// 处理初始化握手
    fn initialize(&mut self) -> Result<()> {
        tracing::info!("Waiting for initialize request");

        let (id, params) = self.connection.initialize_start()?;
        let init_params: InitializeParams = serde_json::from_value(params)?;

        tracing::info!(
            "Received initialize request from client: {:?}",
            init_params.client_info
        );

        let init_result = self.handle_initialize(init_params)?;
        let init_result_json = serde_json::to_value(init_result)?;

        self.connection.initialize_finish(id, init_result_json)?;

        self.state = ServerState::Initialized;
        tracing::info!("LSP server initialized successfully");

        Ok(())
    }

    /// 主事件循环
    ///
    /// 处理来自客户端的所有消息，包括请求、响应和通知
    fn event_loop(&mut self) -> Result<()> {
        tracing::info!("Entering main event loop");

        loop {
            // 检查服务器状态
            if self.state == ServerState::ShuttingDown {
                tracing::info!("Server is shutting down, stopping event loop");
                break;
            }

            // 接收消息
            let msg = match self.connection.receiver.recv() {
                Ok(msg) => msg,
                Err(e) => {
                    tracing::error!("Error receiving message: {}", e);
                    break;
                }
            };

            // 处理消息，捕获错误以保持服务器运行
            if let Err(e) = self.handle_message(msg) {
                tracing::error!("Error handling message: {}", e);
                // 继续运行，不因单个错误而崩溃
            }
        }

        Ok(())
    }

    /// 处理单个消息
    fn handle_message(&mut self, msg: Message) -> Result<()> {
        match msg {
            Message::Request(req) => self.handle_request(req),
            Message::Response(resp) => {
                tracing::debug!("Received response: {:?}", resp.id);
                // 响应消息通常不需要处理
                Ok(())
            }
            Message::Notification(not) => self.handle_notification(not),
        }
    }

    /// 处理请求
    fn handle_request(&mut self, req: Request) -> Result<()> {
        tracing::debug!("Received request: {} (id: {:?})", req.method, req.id);

        // 处理关闭请求
        if self.connection.handle_shutdown(&req)? {
            tracing::info!("Received shutdown request");
            self.state = ServerState::ShuttingDown;
            return Ok(());
        }

        // 根据请求方法分发
        match req.method.as_str() {
            // TODO: 添加其他请求处理器
            // Completion::METHOD => self.handle_completion(req),
            // Hover::METHOD => self.handle_hover(req),
            // GotoDefinition::METHOD => self.handle_goto_definition(req),
            _ => {
                tracing::warn!("Unhandled request method: {}", req.method);
                // 返回方法未实现错误
                self.send_error_response(
                    req.id,
                    lsp_server::ErrorCode::MethodNotFound as i32,
                    format!("Method not found: {}", req.method),
                )?;
            }
        }

        Ok(())
    }

    /// 处理通知
    fn handle_notification(&mut self, not: Notification) -> Result<()> {
        tracing::debug!("Received notification: {}", not.method);

        match not.method.as_str() {
            DidOpenTextDocument::METHOD => {
                let params: DidOpenTextDocumentParams = serde_json::from_value(not.params)?;
                self.handle_did_open(params)?;
            }
            DidChangeTextDocument::METHOD => {
                let params: DidChangeTextDocumentParams = serde_json::from_value(not.params)?;
                self.handle_did_change(params)?;
            }
            DidCloseTextDocument::METHOD => {
                let params: DidCloseTextDocumentParams = serde_json::from_value(not.params)?;
                self.handle_did_close(params)?;
            }
            Exit::METHOD => {
                tracing::info!("Received exit notification");
                self.state = ServerState::ShuttingDown;
            }
            _ => {
                tracing::debug!("Unhandled notification method: {}", not.method);
            }
        }

        Ok(())
    }

    /// 处理文档打开通知
    pub fn handle_did_open(&mut self, params: DidOpenTextDocumentParams) -> Result<()> {
        let doc = params.text_document;
        tracing::info!("Document opened: {}", doc.uri);

        self.document_manager.open(
            doc.uri.clone(),
            doc.version,
            doc.text,
            doc.language_id,
        );

        // TODO: 触发文档分析和诊断

        Ok(())
    }

    /// 处理文档修改通知
    fn handle_did_change(&mut self, params: DidChangeTextDocumentParams) -> Result<()> {
        let uri = params.text_document.uri;
        let version = params.text_document.version;
        tracing::debug!("Document changed: {} (version: {})", uri, version);

        self.document_manager
            .change(&uri, version, params.content_changes);

        // TODO: 触发增量分析和诊断

        Ok(())
    }

    /// 处理文档关闭通知
    fn handle_did_close(&mut self, params: DidCloseTextDocumentParams) -> Result<()> {
        let uri = params.text_document.uri;
        tracing::info!("Document closed: {}", uri);

        self.document_manager.close(&uri);

        // TODO: 清理相关的诊断和缓存

        Ok(())
    }

    /// 处理初始化请求
    ///
    /// 声明服务器支持的所有能力，包括：
    /// - 文档同步（增量更新）
    /// - 智能补全（TOML 配置、宏参数、环境变量）
    /// - 悬停提示（配置文档、宏展开、路由信息）
    /// - 诊断（配置验证、路由验证、依赖注入验证）
    /// - 定义跳转（路由导航）
    /// - 文档符号（路由列表）
    pub fn handle_initialize(&self, _params: InitializeParams) -> Result<InitializeResult> {
        use lsp_types::{
            CompletionOptions, HoverProviderCapability, OneOf, TextDocumentSyncCapability,
            TextDocumentSyncKind, TextDocumentSyncOptions, WorkDoneProgressOptions,
        };

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                // 文档同步能力 - 支持增量更新
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::INCREMENTAL),
                        will_save: None,
                        will_save_wait_until: None,
                        save: None,
                    },
                )),

                // 智能补全能力
                // 支持 TOML 配置项、宏参数、环境变量补全
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(true),
                    trigger_characters: Some(vec![
                        "[".to_string(),  // TOML 配置节
                        ".".to_string(),  // 嵌套配置项
                        "$".to_string(),  // 环境变量
                        "{".to_string(),  // 环境变量插值
                        "#".to_string(),  // 宏属性
                        "(".to_string(),  // 宏参数
                    ]),
                    all_commit_characters: None,
                    work_done_progress_options: WorkDoneProgressOptions {
                        work_done_progress: None,
                    },
                    completion_item: None,
                }),

                // 悬停提示能力
                // 支持配置文档、宏展开、路由信息显示
                hover_provider: Some(HoverProviderCapability::Simple(true)),

                // 定义跳转能力
                // 支持路由路径跳转到处理器函数
                definition_provider: Some(OneOf::Left(true)),

                // 文档符号能力
                // 支持显示文档中的所有路由
                document_symbol_provider: Some(OneOf::Left(true)),

                // 工作空间符号能力
                // 支持全局搜索路由和组件
                workspace_symbol_provider: Some(OneOf::Left(true)),

                // 诊断能力（通过 publishDiagnostics 通知发送）
                // 支持配置验证、路由验证、依赖注入验证

                // 代码操作能力（未来支持快速修复）
                // code_action_provider: Some(CodeActionProviderCapability::Simple(true)),

                // 格式化能力（未来支持 TOML 格式化）
                // document_formatting_provider: Some(OneOf::Left(true)),

                // 重命名能力（未来支持配置项重命名）
                // rename_provider: Some(OneOf::Left(true)),

                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "spring-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    /// 发送错误响应
    fn send_error_response(&self, id: RequestId, code: i32, message: String) -> Result<()> {
        let response = Response {
            id,
            result: None,
            error: Some(lsp_server::ResponseError {
                code,
                message,
                data: None,
            }),
        };

        self.connection
            .sender
            .send(Message::Response(response))
            .map_err(|e| Error::Other(anyhow::anyhow!("Failed to send response: {}", e)))?;

        Ok(())
    }

    /// 优雅关闭服务器
    pub fn shutdown(&mut self) -> Result<()> {
        tracing::info!("Shutting down spring-lsp server");

        // 清理资源
        // TODO: 清理所有缓存、索引等资源

        tracing::info!("Server shutdown complete");
        Ok(())
    }
}

impl Default for LspServer {
    fn default() -> Self {
        Self::start().expect("Failed to start LSP server")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lsp_types::{
        ClientCapabilities, ClientInfo, InitializeParams, TextDocumentItem, Url,
        VersionedTextDocumentIdentifier, WorkDoneProgressParams,
    };

    /// 测试服务器状态转换
    #[test]
    fn test_server_state_transitions() {
        // 初始状态应该是未初始化
        let server = LspServer::start().unwrap();
        assert_eq!(server.state, ServerState::Uninitialized);
    }

    /// 测试文档打开
    #[test]
    fn test_document_open() {
        let mut server = LspServer::start().unwrap();
        server.state = ServerState::Initialized;

        let uri = Url::parse("file:///test.toml").unwrap();
        let params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "toml".to_string(),
                version: 1,
                text: "host = \"localhost\"".to_string(),
            },
        };

        server.handle_did_open(params).unwrap();

        // 验证文档已缓存
        let doc = server.document_manager.get(&uri);
        assert!(doc.is_some());
        let doc = doc.unwrap();
        assert_eq!(doc.version, 1);
        assert_eq!(doc.content, "host = \"localhost\"");
        assert_eq!(doc.language_id, "toml");
    }

    /// 测试文档修改
    #[test]
    fn test_document_change() {
        let mut server = LspServer::start().unwrap();
        server.state = ServerState::Initialized;

        let uri = Url::parse("file:///test.toml").unwrap();

        // 先打开文档
        let open_params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "toml".to_string(),
                version: 1,
                text: "host = \"localhost\"".to_string(),
            },
        };
        server.handle_did_open(open_params).unwrap();

        // 修改文档
        let change_params = DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier {
                uri: uri.clone(),
                version: 2,
            },
            content_changes: vec![lsp_types::TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text: "host = \"127.0.0.1\"".to_string(),
            }],
        };
        server.handle_did_change(change_params).unwrap();

        // 验证文档已更新
        let doc = server.document_manager.get(&uri).unwrap();
        assert_eq!(doc.version, 2);
        assert_eq!(doc.content, "host = \"127.0.0.1\"");
    }

    /// 测试文档关闭
    #[test]
    fn test_document_close() {
        let mut server = LspServer::start().unwrap();
        server.state = ServerState::Initialized;

        let uri = Url::parse("file:///test.toml").unwrap();

        // 先打开文档
        let open_params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "toml".to_string(),
                version: 1,
                text: "host = \"localhost\"".to_string(),
            },
        };
        server.handle_did_open(open_params).unwrap();

        // 验证文档已缓存
        assert!(server.document_manager.get(&uri).is_some());

        // 关闭文档
        let close_params = DidCloseTextDocumentParams {
            text_document: lsp_types::TextDocumentIdentifier { uri: uri.clone() },
        };
        server.handle_did_close(close_params).unwrap();

        // 验证文档已清理
        assert!(server.document_manager.get(&uri).is_none());
    }

    /// 测试初始化响应
    #[test]
    fn test_initialize_response() {
        let server = LspServer::start().unwrap();

        #[allow(deprecated)]
        let params = InitializeParams {
            process_id: Some(1234),
            root_uri: None,
            capabilities: ClientCapabilities::default(),
            client_info: Some(ClientInfo {
                name: "test-client".to_string(),
                version: Some("1.0.0".to_string()),
            }),
            locale: None,
            root_path: None,
            initialization_options: None,
            trace: None,
            workspace_folders: Some(vec![lsp_types::WorkspaceFolder {
                uri: Url::parse("file:///workspace").unwrap(),
                name: "workspace".to_string(),
            }]),
            work_done_progress_params: WorkDoneProgressParams::default(),
        };

        let result = server.handle_initialize(params).unwrap();

        // 验证服务器信息
        assert!(result.server_info.is_some());
        let server_info = result.server_info.unwrap();
        assert_eq!(server_info.name, "spring-lsp");
        assert!(server_info.version.is_some());

        // 验证服务器能力
        let capabilities = result.capabilities;

        // 验证文档同步能力
        assert!(capabilities.text_document_sync.is_some());
        if let Some(lsp_types::TextDocumentSyncCapability::Options(sync_options)) =
            capabilities.text_document_sync
        {
            assert_eq!(sync_options.open_close, Some(true));
            assert_eq!(
                sync_options.change,
                Some(lsp_types::TextDocumentSyncKind::INCREMENTAL)
            );
        } else {
            panic!("Expected TextDocumentSyncOptions");
        }

        // 验证补全能力
        assert!(capabilities.completion_provider.is_some());
        let completion = capabilities.completion_provider.unwrap();
        assert_eq!(completion.resolve_provider, Some(true));
        assert!(completion.trigger_characters.is_some());
        let triggers = completion.trigger_characters.unwrap();
        assert!(triggers.contains(&"[".to_string()));
        assert!(triggers.contains(&"$".to_string()));
        assert!(triggers.contains(&"{".to_string()));

        // 验证悬停能力
        assert!(capabilities.hover_provider.is_some());

        // 验证定义跳转能力
        assert!(capabilities.definition_provider.is_some());

        // 验证文档符号能力
        assert!(capabilities.document_symbol_provider.is_some());

        // 验证工作空间符号能力
        assert!(capabilities.workspace_symbol_provider.is_some());
    }

    /// 测试错误恢复
    #[test]
    fn test_error_recovery() {
        let mut server = LspServer::start().unwrap();
        server.state = ServerState::Initialized;

        // 尝试修改不存在的文档（应该不会崩溃）
        let uri = Url::parse("file:///nonexistent.toml").unwrap();
        let change_params = DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier {
                uri: uri.clone(),
                version: 1,
            },
            content_changes: vec![lsp_types::TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text: "test".to_string(),
            }],
        };

        // 这不应该导致错误，只是不会有任何效果
        let result = server.handle_did_change(change_params);
        assert!(result.is_ok());

        // 验证文档仍然不存在
        assert!(server.document_manager.get(&uri).is_none());
    }
}
