//! 配置系统集成测试
//!
//! 测试配置文件加载、合并和环境变量覆盖功能

use spring_lsp::config::ServerConfig;
use std::env;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_load_workspace_config() {
    // 创建临时工作空间
    let temp_dir = TempDir::new().unwrap();
    let workspace_path = temp_dir.path();

    // 创建工作空间配置文件
    let config_content = r#"
[logging]
level = "debug"
verbose = true

[completion]
trigger_characters = ["[", "."]

[diagnostics]
disabled = ["restful_style"]

[schema]
url = "https://custom.com/schema.json"
"#;

    fs::write(workspace_path.join(".spring-lsp.toml"), config_content).unwrap();

    // 加载配置
    let config = ServerConfig::load(Some(workspace_path));

    // 验证配置
    assert_eq!(config.logging.level, "debug");
    assert!(config.logging.verbose);
    assert_eq!(config.completion.trigger_characters.len(), 2);
    assert!(config.diagnostics.is_disabled("restful_style"));
    assert_eq!(config.schema.url, "https://custom.com/schema.json");
}

#[test]
fn test_env_overrides_workspace_config() {
    // 保存原始环境变量
    let original_level = env::var("SPRING_LSP_LOG_LEVEL").ok();
    let original_schema = env::var("SPRING_LSP_SCHEMA_URL").ok();

    // 创建临时工作空间
    let temp_dir = TempDir::new().unwrap();
    let workspace_path = temp_dir.path();

    // 创建工作空间配置文件
    let config_content = r#"
[logging]
level = "info"

[schema]
url = "https://workspace.com/schema.json"
"#;

    fs::write(workspace_path.join(".spring-lsp.toml"), config_content).unwrap();

    // 设置环境变量
    env::set_var("SPRING_LSP_LOG_LEVEL", "trace");
    env::set_var("SPRING_LSP_SCHEMA_URL", "https://env.com/schema.json");

    // 加载配置
    let config = ServerConfig::load(Some(workspace_path));

    // 验证环境变量覆盖了配置文件
    assert_eq!(config.logging.level, "trace");
    assert_eq!(config.schema.url, "https://env.com/schema.json");

    // 恢复原始环境变量
    match original_level {
        Some(v) => env::set_var("SPRING_LSP_LOG_LEVEL", v),
        None => env::remove_var("SPRING_LSP_LOG_LEVEL"),
    }
    match original_schema {
        Some(v) => env::set_var("SPRING_LSP_SCHEMA_URL", v),
        None => env::remove_var("SPRING_LSP_SCHEMA_URL"),
    }
}

#[test]
fn test_default_config_when_no_file() {
    // 创建空的临时目录
    let temp_dir = TempDir::new().unwrap();
    let workspace_path = temp_dir.path();

    // 加载配置（应该使用默认值）
    let config = ServerConfig::load(Some(workspace_path));

    // 验证默认配置
    assert_eq!(config.logging.level, "info");
    assert!(!config.logging.verbose);
    assert!(config.logging.log_file.is_none());
    assert_eq!(config.completion.trigger_characters.len(), 6);
    assert!(config.diagnostics.disabled.is_empty());
    assert_eq!(
        config.schema.url,
        "https://spring-rs.github.io/config-schema.json"
    );
}

#[test]
fn test_partial_config_uses_defaults() {
    // 创建临时工作空间
    let temp_dir = TempDir::new().unwrap();
    let workspace_path = temp_dir.path();

    // 创建部分配置文件（只配置日志）
    let config_content = r#"
[logging]
level = "warn"
"#;

    fs::write(workspace_path.join(".spring-lsp.toml"), config_content).unwrap();

    // 加载配置
    let config = ServerConfig::load(Some(workspace_path));

    // 验证部分配置生效，其他使用默认值
    assert_eq!(config.logging.level, "warn");
    assert!(!config.logging.verbose); // 默认值
    assert_eq!(config.completion.trigger_characters.len(), 6); // 默认值
    assert!(config.diagnostics.disabled.is_empty()); // 默认值
}

#[test]
fn test_invalid_config_validation() {
    // 创建临时工作空间
    let temp_dir = TempDir::new().unwrap();
    let workspace_path = temp_dir.path();

    // 创建无效配置文件
    let config_content = r#"
[logging]
level = "invalid_level"

[completion]
trigger_characters = []

[schema]
url = "ftp://invalid.com/schema.json"
"#;

    fs::write(workspace_path.join(".spring-lsp.toml"), config_content).unwrap();

    // 加载配置
    let config = ServerConfig::load(Some(workspace_path));

    // 验证配置失败
    assert!(config.validate().is_err());
}

#[test]
fn test_diagnostics_filtering() {
    // 创建临时工作空间
    let temp_dir = TempDir::new().unwrap();
    let workspace_path = temp_dir.path();

    // 创建配置文件，禁用多个诊断类型
    let config_content = r#"
[diagnostics]
disabled = ["deprecated_warning", "restful_style", "circular_dependency"]
"#;

    fs::write(workspace_path.join(".spring-lsp.toml"), config_content).unwrap();

    // 加载配置
    let config = ServerConfig::load(Some(workspace_path));

    // 验证诊断过滤
    assert!(config.diagnostics.is_disabled("deprecated_warning"));
    assert!(config.diagnostics.is_disabled("restful_style"));
    assert!(config.diagnostics.is_disabled("circular_dependency"));
    assert!(!config.diagnostics.is_disabled("type_mismatch"));
}

#[test]
fn test_custom_trigger_characters() {
    // 创建临时工作空间
    let temp_dir = TempDir::new().unwrap();
    let workspace_path = temp_dir.path();

    // 创建配置文件，自定义触发字符
    let config_content = r#"
[completion]
trigger_characters = ["[", "@", ":"]
"#;

    fs::write(workspace_path.join(".spring-lsp.toml"), config_content).unwrap();

    // 加载配置
    let config = ServerConfig::load(Some(workspace_path));

    // 验证自定义触发字符
    assert_eq!(config.completion.trigger_characters.len(), 3);
    assert!(config.completion.trigger_characters.contains(&"[".to_string()));
    assert!(config.completion.trigger_characters.contains(&"@".to_string()));
    assert!(config.completion.trigger_characters.contains(&":".to_string()));
}

#[test]
fn test_local_schema_file() {
    // 创建临时工作空间
    let temp_dir = TempDir::new().unwrap();
    let workspace_path = temp_dir.path();

    // 创建配置文件，使用本地 Schema
    let config_content = r#"
[schema]
url = "file:///opt/spring-lsp/custom-schema.json"
"#;

    fs::write(workspace_path.join(".spring-lsp.toml"), config_content).unwrap();

    // 加载配置
    let config = ServerConfig::load(Some(workspace_path));

    // 验证本地 Schema URL
    assert_eq!(config.schema.url, "file:///opt/spring-lsp/custom-schema.json");
    assert!(config.validate().is_ok());
}
