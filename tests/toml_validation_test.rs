//! TOML 配置验证单元测试

use lsp_types::DiagnosticSeverity;
use spring_lsp::schema::{ConfigSchema, PluginSchema, PropertySchema, TypeInfo};
use spring_lsp::toml_analyzer::TomlAnalyzer;
use spring_lsp::schema::SchemaProvider;
use std::collections::HashMap;

/// 创建测试用的 Schema
fn create_test_schema() -> ConfigSchema {
    let mut plugins = HashMap::new();
    
    // Web 插件配置
    let mut web_properties = HashMap::new();
    web_properties.insert(
        "host".to_string(),
        PropertySchema {
            name: "host".to_string(),
            type_info: TypeInfo::String {
                enum_values: None,
                min_length: Some(1),
                max_length: Some(255),
            },
            description: "Web server host".to_string(),
            default: None,
            required: false,
            deprecated: None,
        },
    );
    web_properties.insert(
        "port".to_string(),
        PropertySchema {
            name: "port".to_string(),
            type_info: TypeInfo::Integer {
                min: Some(1),
                max: Some(65535),
            },
            description: "Web server port".to_string(),
            default: None,
            required: true,
            deprecated: None,
        },
    );
    web_properties.insert(
        "old_config".to_string(),
        PropertySchema {
            name: "old_config".to_string(),
            type_info: TypeInfo::String {
                enum_values: None,
                min_length: None,
                max_length: None,
            },
            description: "Deprecated config".to_string(),
            default: None,
            required: false,
            deprecated: Some("请使用 new_config 代替".to_string()),
        },
    );
    web_properties.insert(
        "mode".to_string(),
        PropertySchema {
            name: "mode".to_string(),
            type_info: TypeInfo::String {
                enum_values: Some(vec!["dev".to_string(), "prod".to_string()]),
                min_length: None,
                max_length: None,
            },
            description: "Server mode".to_string(),
            default: None,
            required: false,
            deprecated: None,
        },
    );
    
    plugins.insert(
        "web".to_string(),
        PluginSchema {
            prefix: "web".to_string(),
            properties: web_properties,
        },
    );
    
    ConfigSchema { plugins }
}

fn create_analyzer_with_schema(schema: ConfigSchema) -> TomlAnalyzer {
    let schema_provider = SchemaProvider::from_schema(schema);
    TomlAnalyzer::new(schema_provider)
}

#[test]
fn test_validate_undefined_section() {
    let schema = create_test_schema();
    let analyzer = create_analyzer_with_schema(schema);
    
    let toml = r#"
[unknown]
key = "value"
"#;
    
    let doc = analyzer.parse(toml).unwrap();
    let diagnostics = analyzer.validate(&doc);
    
    // 应该有一个错误：未定义的配置节
    assert!(!diagnostics.is_empty());
    let error = diagnostics.iter().find(|d| {
        d.severity == Some(DiagnosticSeverity::ERROR)
            && d.message.contains("未在 Schema 中定义")
    });
    assert!(error.is_some(), "应该有未定义配置节的错误");
}

#[test]
fn test_validate_undefined_property() {
    let schema = create_test_schema();
    let analyzer = create_analyzer_with_schema(schema);
    
    let toml = r#"
[web]
unknown_key = "value"
"#;
    
    let doc = analyzer.parse(toml).unwrap();
    let diagnostics = analyzer.validate(&doc);
    
    // 应该有一个错误：未定义的配置项
    assert!(!diagnostics.is_empty());
    let error = diagnostics.iter().find(|d| {
        d.severity == Some(DiagnosticSeverity::ERROR)
            && d.message.contains("未在 Schema 中定义")
    });
    assert!(error.is_some(), "应该有未定义配置项的错误");
}

#[test]
fn test_validate_type_mismatch() {
    let schema = create_test_schema();
    let analyzer = create_analyzer_with_schema(schema);
    
    let toml = r#"
[web]
port = "not_a_number"
"#;
    
    let doc = analyzer.parse(toml).unwrap();
    let diagnostics = analyzer.validate(&doc);
    
    // 应该有一个错误：类型不匹配
    assert!(!diagnostics.is_empty());
    let error = diagnostics.iter().find(|d| {
        d.severity == Some(DiagnosticSeverity::ERROR)
            && d.message.contains("类型不匹配")
    });
    assert!(error.is_some(), "应该有类型不匹配的错误");
}

#[test]
fn test_validate_missing_required_property() {
    let schema = create_test_schema();
    let analyzer = create_analyzer_with_schema(schema);
    
    let toml = r#"
[web]
host = "localhost"
"#;
    
    let doc = analyzer.parse(toml).unwrap();
    let diagnostics = analyzer.validate(&doc);
    
    // 应该有一个警告：缺少必需的配置项 port
    assert!(!diagnostics.is_empty());
    let warning = diagnostics.iter().find(|d| {
        d.severity == Some(DiagnosticSeverity::WARNING)
            && d.message.contains("缺少必需的配置项")
    });
    assert!(warning.is_some(), "应该有缺少必需配置项的警告");
}

#[test]
fn test_validate_deprecated_property() {
    let schema = create_test_schema();
    let analyzer = create_analyzer_with_schema(schema);
    
    let toml = r#"
[web]
port = 8080
old_config = "value"
"#;
    
    let doc = analyzer.parse(toml).unwrap();
    let diagnostics = analyzer.validate(&doc);
    
    // 应该有一个警告：使用了废弃的配置项
    assert!(!diagnostics.is_empty());
    let warning = diagnostics.iter().find(|d| {
        d.severity == Some(DiagnosticSeverity::WARNING)
            && d.message.contains("已废弃")
    });
    assert!(warning.is_some(), "应该有废弃配置项的警告");
}

#[test]
fn test_validate_env_var_empty_name() {
    let schema = create_test_schema();
    let analyzer = create_analyzer_with_schema(schema);
    
    let toml = r#"
[web]
host = "${}"
port = 8080
"#;
    
    let doc = analyzer.parse(toml).unwrap();
    let diagnostics = analyzer.validate(&doc);
    
    // 应该有一个错误：环境变量名为空
    assert!(!diagnostics.is_empty());
    let error = diagnostics.iter().find(|d| {
        d.severity == Some(DiagnosticSeverity::ERROR)
            && d.message.contains("环境变量名不能为空")
    });
    assert!(error.is_some(), "应该有环境变量名为空的错误");
}

#[test]
fn test_validate_env_var_invalid_name() {
    let schema = create_test_schema();
    let analyzer = create_analyzer_with_schema(schema);
    
    let toml = r#"
[web]
host = "${invalid-name}"
port = 8080
"#;
    
    let doc = analyzer.parse(toml).unwrap();
    let diagnostics = analyzer.validate(&doc);
    
    // 应该有一个警告：环境变量名不符合命名规范
    assert!(!diagnostics.is_empty());
    let warning = diagnostics.iter().find(|d| {
        d.severity == Some(DiagnosticSeverity::WARNING)
            && d.message.contains("不符合命名规范")
    });
    assert!(warning.is_some(), "应该有环境变量命名规范的警告");
}

#[test]
fn test_validate_integer_out_of_range_min() {
    let schema = create_test_schema();
    let analyzer = create_analyzer_with_schema(schema);
    
    let toml = r#"
[web]
port = 0
"#;
    
    let doc = analyzer.parse(toml).unwrap();
    let diagnostics = analyzer.validate(&doc);
    
    // 应该有一个错误：值小于最小值
    assert!(!diagnostics.is_empty());
    let error = diagnostics.iter().find(|d| {
        d.severity == Some(DiagnosticSeverity::ERROR)
            && d.message.contains("小于最小值")
    });
    assert!(error.is_some(), "应该有值小于最小值的错误");
}

#[test]
fn test_validate_integer_out_of_range_max() {
    let schema = create_test_schema();
    let analyzer = create_analyzer_with_schema(schema);
    
    let toml = r#"
[web]
port = 70000
"#;
    
    let doc = analyzer.parse(toml).unwrap();
    let diagnostics = analyzer.validate(&doc);
    
    // 应该有一个错误：值超过最大值
    assert!(!diagnostics.is_empty());
    let error = diagnostics.iter().find(|d| {
        d.severity == Some(DiagnosticSeverity::ERROR)
            && d.message.contains("超过最大值")
    });
    assert!(error.is_some(), "应该有值超过最大值的错误");
}

#[test]
fn test_validate_string_too_short() {
    let schema = create_test_schema();
    let analyzer = create_analyzer_with_schema(schema);
    
    let toml = r#"
[web]
host = ""
port = 8080
"#;
    
    let doc = analyzer.parse(toml).unwrap();
    let diagnostics = analyzer.validate(&doc);
    
    // 应该有一个错误：字符串长度小于最小长度
    assert!(!diagnostics.is_empty());
    let error = diagnostics.iter().find(|d| {
        d.severity == Some(DiagnosticSeverity::ERROR)
            && d.message.contains("小于最小长度")
    });
    assert!(error.is_some(), "应该有字符串长度小于最小长度的错误");
}

#[test]
fn test_validate_invalid_enum_value() {
    let schema = create_test_schema();
    let analyzer = create_analyzer_with_schema(schema);
    
    let toml = r#"
[web]
port = 8080
mode = "invalid"
"#;
    
    let doc = analyzer.parse(toml).unwrap();
    let diagnostics = analyzer.validate(&doc);
    
    // 应该有一个错误：枚举值无效
    assert!(!diagnostics.is_empty());
    let error = diagnostics.iter().find(|d| {
        d.severity == Some(DiagnosticSeverity::ERROR)
            && d.message.contains("不在允许的枚举值中")
    });
    assert!(error.is_some(), "应该有枚举值无效的错误");
}

#[test]
fn test_validate_valid_config() {
    let schema = create_test_schema();
    let analyzer = create_analyzer_with_schema(schema);
    
    let toml = r#"
[web]
host = "localhost"
port = 8080
mode = "dev"
"#;
    
    let doc = analyzer.parse(toml).unwrap();
    let diagnostics = analyzer.validate(&doc);
    
    // 不应该有任何错误或警告
    let errors = diagnostics.iter().filter(|d| {
        d.severity == Some(DiagnosticSeverity::ERROR)
    }).count();
    assert_eq!(errors, 0, "有效的配置不应该有错误");
}

#[test]
fn test_validate_valid_env_var() {
    let schema = create_test_schema();
    let analyzer = create_analyzer_with_schema(schema);
    
    let toml = r#"
[web]
host = "${HOST:localhost}"
port = 8080
"#;
    
    let doc = analyzer.parse(toml).unwrap();
    let diagnostics = analyzer.validate(&doc);
    
    // 不应该有环境变量相关的错误
    let env_errors = diagnostics.iter().filter(|d| {
        d.message.contains("环境变量")
    }).count();
    assert_eq!(env_errors, 0, "有效的环境变量不应该有错误");
}

#[test]
fn test_validate_multiple_errors() {
    let schema = create_test_schema();
    let analyzer = create_analyzer_with_schema(schema);
    
    let toml = r#"
[web]
unknown_key = "value"
port = "not_a_number"
mode = "invalid"
"#;
    
    let doc = analyzer.parse(toml).unwrap();
    let diagnostics = analyzer.validate(&doc);
    
    // 应该有多个错误
    let errors = diagnostics.iter().filter(|d| {
        d.severity == Some(DiagnosticSeverity::ERROR)
    }).count();
    assert!(errors >= 3, "应该有至少 3 个错误，实际: {}", errors);
}
