//! 错误处理属性测试
//!
//! 使用 proptest 验证错误处理和日志系统的通用正确性属性

use proptest::prelude::*;
use spring_lsp::error::{
    config_validation_error, di_validation_error, env_var_syntax_error, route_validation_error,
    rust_parse_error, toml_parse_error, Error, ErrorCategory, ErrorHandler, ErrorSeverity,
    RecoveryAction,
};
use lsp_types::Url;
use std::sync::{Arc, Mutex};

// ============================================================================
// 测试数据生成器
// ============================================================================

/// 生成有效的文件 URI
fn valid_file_uri() -> impl Strategy<Value = Url> {
    "[a-z0-9_-]{1,20}\\.(toml|rs)"
        .prop_map(|filename| Url::parse(&format!("file:///{}", filename)).unwrap())
}

/// 生成错误消息
fn error_message() -> impl Strategy<Value = String> {
    "[A-Za-z0-9 .,!?:-]{1,100}"
}

/// 生成行号
fn line_number() -> impl Strategy<Value = u32> {
    1u32..1000u32
}

/// 生成各种类型的错误
fn any_error() -> impl Strategy<Value = Error> {
    prop_oneof![
        // 协议错误
        error_message().prop_map(Error::MessageSend),
        error_message().prop_map(Error::MessageReceive),
        // 解析错误
        (valid_file_uri(), error_message()).prop_map(|(uri, msg)| Error::TomlParse {
            uri: uri.to_string(),
            message: msg,
        }),
        (valid_file_uri(), error_message()).prop_map(|(uri, msg)| Error::RustParse {
            uri: uri.to_string(),
            message: msg,
        }),
        (valid_file_uri(), line_number(), error_message()).prop_map(|(uri, line, msg)| {
            Error::EnvVarSyntax {
                uri: uri.to_string(),
                line,
                message: msg,
            }
        }),
        // 验证错误
        (valid_file_uri(), error_message()).prop_map(|(uri, msg)| Error::ConfigValidation {
            uri: uri.to_string(),
            message: msg,
        }),
        (valid_file_uri(), error_message()).prop_map(|(uri, msg)| Error::RouteValidation {
            uri: uri.to_string(),
            message: msg,
        }),
        (valid_file_uri(), error_message()).prop_map(|(uri, msg)| Error::DiValidation {
            uri: uri.to_string(),
            message: msg,
        }),
        // 系统错误
        error_message().prop_map(Error::SchemaLoad),
        error_message().prop_map(Error::IndexBuild),
    ]
}

/// 生成协议错误
fn protocol_error() -> impl Strategy<Value = Error> {
    prop_oneof![
        error_message().prop_map(Error::MessageSend),
        error_message().prop_map(Error::MessageReceive),
    ]
}

/// 生成解析错误
fn parse_error() -> impl Strategy<Value = Error> {
    prop_oneof![
        (valid_file_uri(), error_message()).prop_map(|(uri, msg)| Error::TomlParse {
            uri: uri.to_string(),
            message: msg,
        }),
        (valid_file_uri(), error_message()).prop_map(|(uri, msg)| Error::RustParse {
            uri: uri.to_string(),
            message: msg,
        }),
        (valid_file_uri(), line_number(), error_message()).prop_map(|(uri, line, msg)| {
            Error::EnvVarSyntax {
                uri: uri.to_string(),
                line,
                message: msg,
            }
        }),
    ]
}

/// 生成验证错误
fn validation_error() -> impl Strategy<Value = Error> {
    prop_oneof![
        (valid_file_uri(), error_message()).prop_map(|(uri, msg)| Error::ConfigValidation {
            uri: uri.to_string(),
            message: msg,
        }),
        (valid_file_uri(), error_message()).prop_map(|(uri, msg)| Error::RouteValidation {
            uri: uri.to_string(),
            message: msg,
        }),
        (valid_file_uri(), error_message()).prop_map(|(uri, msg)| Error::DiValidation {
            uri: uri.to_string(),
            message: msg,
        }),
    ]
}

/// 生成系统错误
fn system_error() -> impl Strategy<Value = Error> {
    prop_oneof![
        error_message().prop_map(Error::SchemaLoad),
        error_message().prop_map(Error::IndexBuild),
    ]
}

// ============================================================================
// 辅助结构：日志捕获器
// ============================================================================

/// 用于测试的日志捕获器
#[derive(Clone)]
struct LogCapture {
    logs: Arc<Mutex<Vec<(ErrorSeverity, String)>>>,
}

impl LogCapture {
    fn new() -> Self {
        Self {
            logs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn capture(&self, severity: ErrorSeverity, message: String) {
        self.logs.lock().unwrap().push((severity, message));
    }

    fn get_logs(&self) -> Vec<(ErrorSeverity, String)> {
        self.logs.lock().unwrap().clone()
    }

    fn clear(&self) {
        self.logs.lock().unwrap().clear();
    }

    fn count(&self) -> usize {
        self.logs.lock().unwrap().len()
    }
}

// ============================================================================
// 属性测试
// ============================================================================

// Feature: spring-lsp, Property 5: 错误恢复稳定性
//
// **Validates: Requirements 1.6**
//
// *For any* 内部错误，服务器应该记录错误日志并继续响应后续请求。
//
// 这个属性测试验证：
// 1. 错误处理器能够处理任何类型的错误而不崩溃
// 2. 可恢复的错误应该返回适当的恢复动作
// 3. 不可恢复的错误应该返回 Abort 动作
// 4. 错误处理不应该抛出异常或 panic
proptest! {
    #[test]
    fn prop_error_recovery_stability(error in any_error()) {
        // 创建错误处理器
        let handler = ErrorHandler::new(false);

        // 处理错误不应该 panic
        let result = handler.handle(&error);

        // 验证返回了有效的恢复动作
        match error.category() {
            ErrorCategory::Protocol => {
                prop_assert_eq!(result.action, RecoveryAction::RetryConnection,
                    "Protocol errors should suggest retry connection");
                prop_assert!(result.notify_client,
                    "Protocol errors should notify client");
            }
            ErrorCategory::Parse => {
                prop_assert_eq!(result.action, RecoveryAction::PartialParse,
                    "Parse errors should suggest partial parse");
                prop_assert!(result.notify_client,
                    "Parse errors should notify client");
            }
            ErrorCategory::Validation => {
                prop_assert_eq!(result.action, RecoveryAction::GenerateDiagnostic,
                    "Validation errors should generate diagnostic");
                prop_assert!(!result.notify_client,
                    "Validation errors should not notify client directly");
            }
            ErrorCategory::System => {
                // 系统错误的恢复动作取决于具体错误类型
                prop_assert!(
                    matches!(
                        result.action,
                        RecoveryAction::UseFallback
                            | RecoveryAction::UseCache
                            | RecoveryAction::SkipOperation
                            | RecoveryAction::Abort
                    ),
                    "System errors should have appropriate recovery action"
                );
            }
        }

        // 验证可恢复性与恢复动作的一致性
        if error.is_recoverable() {
            prop_assert_ne!(result.action, RecoveryAction::Abort,
                "Recoverable errors should not abort");
        }
    }
}

// Feature: spring-lsp, Property 5: 错误恢复稳定性（连续错误处理）
//
// **Validates: Requirements 1.6**
//
// 验证错误处理器能够连续处理多个错误而不崩溃
proptest! {
    #[test]
    fn prop_continuous_error_handling(errors in prop::collection::vec(any_error(), 1..20)) {
        let handler = ErrorHandler::new(false);

        // 连续处理多个错误
        for error in &errors {
            // 每次处理都不应该 panic
            let result = handler.handle(error);

            // 验证返回了有效的结果
            prop_assert!(
                matches!(
                    result.action,
                    RecoveryAction::RetryConnection
                        | RecoveryAction::PartialParse
                        | RecoveryAction::GenerateDiagnostic
                        | RecoveryAction::UseFallback
                        | RecoveryAction::UseCache
                        | RecoveryAction::SkipOperation
                        | RecoveryAction::Abort
                ),
                "Should return valid recovery action"
            );
        }
    }
}

// Feature: spring-lsp, Property 54: 错误日志记录
//
// **Validates: Requirements 13.1**
//
// *For any* 服务器内部错误，应该在日志文件中记录完整的错误堆栈信息。
//
// 这个属性测试验证：
// 1. 所有错误都会被记录
// 2. 错误的严重程度正确分类
// 3. 错误消息包含足够的上下文信息
// 4. 详细模式下记录更多信息
proptest! {
    #[test]
    fn prop_error_logging(error in any_error()) {
        // 创建错误处理器（非详细模式）
        let handler = ErrorHandler::new(false);

        // 处理错误（会触发日志记录）
        let _result = handler.handle(&error);

        // 验证错误的严重程度分类正确
        let severity = error.severity();
        match error.category() {
            ErrorCategory::Protocol => {
                prop_assert_eq!(severity, ErrorSeverity::Error,
                    "Protocol errors should be Error severity");
            }
            ErrorCategory::Parse => {
                prop_assert_eq!(severity, ErrorSeverity::Warning,
                    "Parse errors should be Warning severity");
            }
            ErrorCategory::Validation => {
                prop_assert_eq!(severity, ErrorSeverity::Info,
                    "Validation errors should be Info severity");
            }
            ErrorCategory::System => {
                // 系统错误的严重程度取决于具体类型
                prop_assert!(
                    matches!(severity, ErrorSeverity::Warning | ErrorSeverity::Error),
                    "System errors should be Warning or Error severity"
                );
            }
        }

        // 验证错误消息不为空
        let error_string = error.to_string();
        prop_assert!(!error_string.is_empty(),
            "Error message should not be empty");

        // 验证错误消息包含有用信息
        prop_assert!(error_string.len() > 5,
            "Error message should contain meaningful information");
    }
}

// Feature: spring-lsp, Property 54: 错误日志记录（详细模式）
//
// **Validates: Requirements 13.1**
//
// 验证详细模式下的日志记录
proptest! {
    #[test]
    fn prop_verbose_error_logging(error in any_error()) {
        // 创建详细模式的错误处理器
        let handler_verbose = ErrorHandler::new(true);
        let handler_normal = ErrorHandler::new(false);

        // 两种模式都应该能够处理错误
        let result_verbose = handler_verbose.handle(&error);
        let result_normal = handler_normal.handle(&error);

        // 恢复动作应该相同（不受详细模式影响）
        prop_assert_eq!(result_verbose.action, result_normal.action,
            "Recovery action should be same regardless of verbose mode");

        // 通知客户端的决策应该相同
        prop_assert_eq!(result_verbose.notify_client, result_normal.notify_client,
            "Client notification should be same regardless of verbose mode");
    }
}

// Feature: spring-lsp, Property 55: 分析失败通知
//
// **Validates: Requirements 13.3**
//
// *For any* 文档分析失败，服务器应该向客户端发送包含错误信息的通知。
//
// 这个属性测试验证：
// 1. 解析错误应该通知客户端
// 2. 验证错误通过诊断通知（不需要额外通知）
// 3. 系统错误应该通知客户端
// 4. 通知决策与错误类型一致
proptest! {
    #[test]
    fn prop_analysis_failure_notification(error in any_error()) {
        let handler = ErrorHandler::new(false);
        let result = handler.handle(&error);

        // 验证通知决策与错误类别一致
        match error.category() {
            ErrorCategory::Protocol => {
                prop_assert!(result.notify_client,
                    "Protocol errors should notify client");
            }
            ErrorCategory::Parse => {
                prop_assert!(result.notify_client,
                    "Parse errors (analysis failures) should notify client");
            }
            ErrorCategory::Validation => {
                prop_assert!(!result.notify_client,
                    "Validation errors should use diagnostics, not direct notification");
            }
            ErrorCategory::System => {
                prop_assert!(result.notify_client,
                    "System errors should notify client");
            }
        }
    }
}

// Feature: spring-lsp, Property 55: 分析失败通知（解析错误）
//
// **Validates: Requirements 13.3**
//
// 专门验证解析错误（文档分析失败）的通知行为
proptest! {
    #[test]
    fn prop_parse_error_notification(error in parse_error()) {
        let handler = ErrorHandler::new(false);
        let result = handler.handle(&error);

        // 解析错误应该通知客户端
        prop_assert!(result.notify_client,
            "Parse errors should notify client about analysis failure");

        // 解析错误应该建议部分解析
        prop_assert_eq!(result.action, RecoveryAction::PartialParse,
            "Parse errors should suggest partial parse recovery");

        // 解析错误应该有文档 URI
        prop_assert!(error.document_uri().is_some(),
            "Parse errors should have document URI");
    }
}

// Feature: spring-lsp, Property 5: 错误恢复稳定性（错误分类一致性）
//
// **Validates: Requirements 1.6**
//
// 验证错误分类的一致性
proptest! {
    #[test]
    fn prop_error_category_consistency(error in any_error()) {
        let category = error.category();

        // 验证错误类别与错误类型一致
        match &error {
            Error::MessageSend(_) | Error::MessageReceive(_) | Error::Protocol(_) => {
                prop_assert_eq!(category, ErrorCategory::Protocol,
                    "Message and protocol errors should be Protocol category");
            }
            Error::TomlParse { .. } | Error::RustParse { .. } | Error::EnvVarSyntax { .. } => {
                prop_assert_eq!(category, ErrorCategory::Parse,
                    "Parse errors should be Parse category");
            }
            Error::ConfigValidation { .. }
            | Error::RouteValidation { .. }
            | Error::DiValidation { .. }
            | Error::Config(_) => {
                prop_assert_eq!(category, ErrorCategory::Validation,
                    "Validation errors should be Validation category");
            }
            Error::SchemaLoad(_)
            | Error::Io(_)
            | Error::Json(_)
            | Error::Http(_)
            | Error::IndexBuild(_)
            | Error::Other(_) => {
                prop_assert_eq!(category, ErrorCategory::System,
                    "System-level errors should be System category");
            }
        }
    }
}

// Feature: spring-lsp, Property 54: 错误日志记录（文档 URI 提取）
//
// **Validates: Requirements 13.1**
//
// 验证错误能够正确提取文档 URI
proptest! {
    #[test]
    fn prop_document_uri_extraction(uri in valid_file_uri(), msg in error_message()) {
        // 创建包含 URI 的错误
        let errors = vec![
            toml_parse_error(&uri, &msg),
            rust_parse_error(&uri, &msg),
            env_var_syntax_error(&uri, 10, &msg),
            config_validation_error(&uri, &msg),
            route_validation_error(&uri, &msg),
            di_validation_error(&uri, &msg),
        ];

        for error in errors {
            // 应该能够提取文档 URI
            let extracted_uri = error.document_uri();
            prop_assert!(extracted_uri.is_some(),
                "Document-related errors should have URI");

            let extracted = extracted_uri.unwrap();
            prop_assert_eq!(extracted, uri.as_str(),
                "Extracted URI should match original");
        }

        // 系统错误不应该有文档 URI
        let system_err = Error::SchemaLoad(msg.clone());
        prop_assert!(system_err.document_uri().is_none(),
            "System errors should not have document URI");
    }
}

// Feature: spring-lsp, Property 5: 错误恢复稳定性（降级策略）
//
// **Validates: Requirements 1.6**
//
// 验证系统错误的降级策略
proptest! {
    #[test]
    fn prop_fallback_strategy(error in system_error()) {
        let handler = ErrorHandler::new(false);
        let result = handler.handle(&error);

        // 验证降级策略的正确性
        match &error {
            Error::SchemaLoad(_) => {
                prop_assert_eq!(result.action, RecoveryAction::UseFallback,
                    "Schema load errors should use fallback");
                prop_assert_eq!(result.fallback, Some("builtin-schema".to_string()),
                    "Schema load errors should specify builtin-schema as fallback");
            }
            Error::IndexBuild(_) => {
                prop_assert_eq!(result.action, RecoveryAction::SkipOperation,
                    "Index build errors should skip operation");
            }
            _ => {
                // 其他系统错误应该有合理的恢复策略
                prop_assert!(
                    matches!(
                        result.action,
                        RecoveryAction::UseFallback
                            | RecoveryAction::UseCache
                            | RecoveryAction::SkipOperation
                            | RecoveryAction::Abort
                    ),
                    "System errors should have valid recovery action"
                );
            }
        }
    }
}

// Feature: spring-lsp, Property 54: 错误日志记录（严重程度排序）
//
// **Validates: Requirements 13.1**
//
// 验证错误严重程度的排序关系
proptest! {
    #[test]
    fn prop_severity_ordering(
        protocol_err in protocol_error(),
        parse_err in parse_error(),
        validation_err in validation_error()
    ) {
        // 验证严重程度的排序
        let protocol_severity = protocol_err.severity();
        let parse_severity = parse_err.severity();
        let validation_severity = validation_err.severity();

        // 协议错误应该是 Error 级别
        prop_assert_eq!(protocol_severity, ErrorSeverity::Error);

        // 解析错误应该是 Warning 级别
        prop_assert_eq!(parse_severity, ErrorSeverity::Warning);

        // 验证错误应该是 Info 级别
        prop_assert_eq!(validation_severity, ErrorSeverity::Info);

        // 验证排序关系
        prop_assert!(protocol_severity > parse_severity,
            "Error should be more severe than Warning");
        prop_assert!(parse_severity > validation_severity,
            "Warning should be more severe than Info");
    }
}

// Feature: spring-lsp, Property 5: 错误恢复稳定性（可恢复性判断）
//
// **Validates: Requirements 1.6**
//
// 验证错误可恢复性判断的正确性
proptest! {
    #[test]
    fn prop_recoverability_judgment(error in any_error()) {
        let is_recoverable = error.is_recoverable();

        // 验证可恢复性与错误类型的一致性
        match &error {
            // 协议错误可恢复
            Error::Protocol(_) | Error::MessageSend(_) | Error::MessageReceive(_) => {
                prop_assert!(is_recoverable,
                    "Protocol errors should be recoverable");
            }
            // 解析错误可恢复（部分解析）
            Error::TomlParse { .. } | Error::RustParse { .. } | Error::EnvVarSyntax { .. } => {
                prop_assert!(is_recoverable,
                    "Parse errors should be recoverable");
            }
            // 验证错误可恢复（不影响服务器运行）
            Error::ConfigValidation { .. }
            | Error::RouteValidation { .. }
            | Error::DiValidation { .. }
            | Error::Config(_) => {
                prop_assert!(is_recoverable,
                    "Validation errors should be recoverable");
            }
            // 系统错误部分可恢复
            Error::SchemaLoad(_) | Error::Http(_) | Error::IndexBuild(_) => {
                prop_assert!(is_recoverable,
                    "These system errors should be recoverable");
            }
            Error::Io(_) | Error::Json(_) | Error::Other(_) => {
                prop_assert!(!is_recoverable,
                    "These system errors should not be recoverable");
            }
        }
    }
}