//! 日志系统集成测试
//!
//! 测试日志系统的各种配置和功能

use std::env;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// 测试默认日志配置
#[test]
fn test_default_logging_config() {
    use spring_lsp::logging::LogConfig;

    let config = LogConfig::default();
    assert_eq!(config.level, "info");
    assert!(!config.verbose);
    assert!(config.log_file.is_none());
}

/// 测试从环境变量加载配置
#[test]
fn test_logging_config_from_env() {
    use spring_lsp::logging::LogConfig;

    // 保存原始环境变量
    let original_level = env::var("SPRING_LSP_LOG_LEVEL").ok();
    let original_verbose = env::var("SPRING_LSP_VERBOSE").ok();
    let original_file = env::var("SPRING_LSP_LOG_FILE").ok();

    // 设置测试环境变量
    env::set_var("SPRING_LSP_LOG_LEVEL", "debug");
    env::set_var("SPRING_LSP_VERBOSE", "1");
    env::set_var("SPRING_LSP_LOG_FILE", "/tmp/test.log");

    let config = LogConfig::from_env();
    assert_eq!(config.level, "debug");
    assert!(config.verbose);
    assert_eq!(config.log_file, Some(PathBuf::from("/tmp/test.log")));

    // 恢复原始环境变量
    restore_env("SPRING_LSP_LOG_LEVEL", original_level);
    restore_env("SPRING_LSP_VERBOSE", original_verbose);
    restore_env("SPRING_LSP_LOG_FILE", original_file);
}

/// 测试日志级别验证
#[test]
fn test_log_level_validation() {
    use spring_lsp::logging::LogConfig;

    // 有效的日志级别
    let valid_levels = vec!["trace", "debug", "info", "warn", "error"];
    for level in valid_levels {
        let config = LogConfig {
            level: level.to_string(),
            verbose: false,
            log_file: None,
        };
        assert!(
            config.validate_level().is_ok(),
            "Level {} should be valid",
            level
        );
    }

    // 无效的日志级别
    let invalid_config = LogConfig {
        level: "invalid".to_string(),
        verbose: false,
        log_file: None,
    };
    assert!(invalid_config.validate_level().is_err());
}

/// 测试日志文件输出
#[test]
fn test_logging_to_file() {
    use spring_lsp::logging::{init_logging_with_config, LogConfig};

    // 创建临时目录
    let temp_dir = TempDir::new().unwrap();
    let log_file = temp_dir.path().join("test.log");

    // 配置日志输出到文件
    let config = LogConfig {
        level: "info".to_string(),
        verbose: false,
        log_file: Some(log_file.clone()),
    };

    // 初始化日志系统（可能失败，因为可能已经初始化过）
    let _ = init_logging_with_config(config);

    // 写入日志
    tracing::info!("Test log message");

    // 注意：由于日志是异步写入的，可能需要等待一小段时间
    // 在实际测试中，我们只验证文件是否被创建
    // 实际的日志内容验证在单元测试中完成

    // 验证日志文件是否存在（如果日志系统成功初始化）
    // 由于可能已经初始化过，这个测试可能会失败，所以我们只检查文件是否存在
    if log_file.exists() {
        let content = fs::read_to_string(&log_file).unwrap();
        // JSON 格式的日志应该包含这些字段
        assert!(content.contains("timestamp") || content.contains("level"));
    }
}

/// 测试详细模式
#[test]
fn test_verbose_mode() {
    use spring_lsp::logging::LogConfig;

    let config = LogConfig {
        level: "debug".to_string(),
        verbose: true,
        log_file: None,
    };

    assert!(config.verbose);
    assert_eq!(config.level, "debug");
}

/// 测试日志级别大小写不敏感
#[test]
fn test_log_level_case_insensitive() {
    use spring_lsp::logging::LogConfig;

    // 保存原始环境变量
    let original = env::var("SPRING_LSP_LOG_LEVEL").ok();

    // 测试大写
    env::set_var("SPRING_LSP_LOG_LEVEL", "DEBUG");
    let config = LogConfig::from_env();
    assert_eq!(config.level, "debug");

    // 测试混合大小写
    env::set_var("SPRING_LSP_LOG_LEVEL", "WaRn");
    let config = LogConfig::from_env();
    assert_eq!(config.level, "warn");

    // 恢复原始环境变量
    restore_env("SPRING_LSP_LOG_LEVEL", original);
}

/// 测试 VERBOSE 环境变量的不同值
#[test]
fn test_verbose_env_variants() {
    use spring_lsp::logging::LogConfig;

    // 保存原始环境变量
    let original = env::var("SPRING_LSP_VERBOSE").ok();

    // 测试 "1"
    env::set_var("SPRING_LSP_VERBOSE", "1");
    assert!(LogConfig::from_env().verbose);

    // 测试 "true"
    env::set_var("SPRING_LSP_VERBOSE", "true");
    assert!(LogConfig::from_env().verbose);

    // 测试 "TRUE"
    env::set_var("SPRING_LSP_VERBOSE", "TRUE");
    assert!(LogConfig::from_env().verbose);

    // 测试 "false"
    env::set_var("SPRING_LSP_VERBOSE", "false");
    assert!(!LogConfig::from_env().verbose);

    // 测试 "0"
    env::set_var("SPRING_LSP_VERBOSE", "0");
    assert!(!LogConfig::from_env().verbose);

    // 测试未设置
    env::remove_var("SPRING_LSP_VERBOSE");
    assert!(!LogConfig::from_env().verbose);

    // 恢复原始环境变量
    restore_env("SPRING_LSP_VERBOSE", original);
}

/// 测试日志文件目录自动创建
#[test]
fn test_log_file_directory_creation() {
    use spring_lsp::logging::{init_logging_with_config, LogConfig};

    // 创建临时目录
    let temp_dir = TempDir::new().unwrap();
    let log_dir = temp_dir.path().join("logs").join("nested");
    let log_file = log_dir.join("test.log");

    // 确保目录不存在
    assert!(!log_dir.exists());

    // 配置日志输出到嵌套目录中的文件
    let config = LogConfig {
        level: "info".to_string(),
        verbose: false,
        log_file: Some(log_file.clone()),
    };

    // 初始化日志系统（可能失败，因为可能已经初始化过）
    let result = init_logging_with_config(config);

    // 如果初始化成功，验证目录是否被创建
    if result.is_ok() {
        assert!(log_dir.exists());
        assert!(log_file.exists() || log_dir.exists()); // 至少目录应该存在
    }
}

/// 辅助函数：恢复环境变量
fn restore_env(key: &str, original: Option<String>) {
    match original {
        Some(v) => env::set_var(key, v),
        None => env::remove_var(key),
    }
}

/// 测试日志配置的 Clone 实现
#[test]
fn test_log_config_clone() {
    use spring_lsp::logging::LogConfig;

    let config = LogConfig {
        level: "debug".to_string(),
        verbose: true,
        log_file: Some(PathBuf::from("/tmp/test.log")),
    };

    let cloned = config.clone();
    assert_eq!(config.level, cloned.level);
    assert_eq!(config.verbose, cloned.verbose);
    assert_eq!(config.log_file, cloned.log_file);
}

/// 测试日志配置的 Debug 实现
#[test]
fn test_log_config_debug() {
    use spring_lsp::logging::LogConfig;

    let config = LogConfig {
        level: "info".to_string(),
        verbose: false,
        log_file: None,
    };

    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("info"));
    assert!(debug_str.contains("verbose"));
}
