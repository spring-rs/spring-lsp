//! TOML 配置验证属性测试
//! 
//! 使用 proptest 验证配置验证功能在随机生成的输入下的正确性

use proptest::prelude::*;
use lsp_types::DiagnosticSeverity;
use spring_lsp::schema::{ConfigSchema, PluginSchema, PropertySchema, TypeInfo, SchemaProvider};
use spring_lsp::toml_analyzer::TomlAnalyzer;
use std::collections::HashMap;

// ============================================================================
// 辅助函数：创建测试 Schema
// ============================================================================

/// 创建包含指定插件的 Schema
fn create_schema_with_plugin(
    plugin_name: &str,
    properties: HashMap<String, PropertySchema>,
) -> ConfigSchema {
    let mut plugins = HashMap::new();
    plugins.insert(
        plugin_name.to_string(),
        PluginSchema {
            prefix: plugin_name.to_string(),
            properties,
        },
    );
    ConfigSchema { plugins }
}

/// 创建分析器
fn create_analyzer_with_schema(schema: ConfigSchema) -> TomlAnalyzer {
    let schema_provider = SchemaProvider::from_schema(schema);
    TomlAnalyzer::new(schema_provider)
}

// ============================================================================
// 测试策略：生成有效的配置元素
// ============================================================================

/// 生成有效的配置节名称
fn valid_section_name() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_-]{0,20}"
}

/// 生成有效的配置键名
fn valid_property_key() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,20}"
}

/// 生成有效的字符串值
fn valid_string_value() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9_./:-]{1,50}"
}

/// 生成有效的环境变量名
fn valid_env_var_name() -> impl Strategy<Value = String> {
    "[A-Z][A-Z0-9_]{0,20}"
}

/// 生成无效的环境变量名（包含小写字母或特殊字符）
fn invalid_env_var_name() -> impl Strategy<Value = String> {
    prop::sample::select(vec![
        "invalid-name".to_string(),
        "invalid.name".to_string(),
        "invalid name".to_string(),
        "invalidName".to_string(),
        "123invalid".to_string(),
    ])
}

/// 生成整数值
fn integer_value() -> impl Strategy<Value = i64> {
    -10000i64..10000i64
}

/// 生成浮点数值
fn float_value() -> impl Strategy<Value = f64> {
    -10000.0f64..10000.0f64
}

// ============================================================================
// Property 19: 配置项定义验证
// **Validates: Requirements 5.1**
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    /// Feature: spring-lsp, Property 19: 配置项定义验证
    /// 
    /// For any 配置文件中的配置项，如果该配置项不在 Schema 中定义，验证器应该生成错误诊断。
    #[test]
    fn prop_undefined_property_generates_error(
        section_name in valid_section_name(),
        defined_key in valid_property_key(),
        undefined_key in valid_property_key(),
        value in valid_string_value()
    ) {
        // 确保 undefined_key 与 defined_key 不同
        prop_assume!(defined_key != undefined_key);
        
        // 创建只包含 defined_key 的 Schema
        let mut properties = HashMap::new();
        properties.insert(
            defined_key.clone(),
            PropertySchema {
                name: defined_key.clone(),
                type_info: TypeInfo::String {
                    enum_values: None,
                    min_length: None,
                    max_length: None,
                },
                description: "Test property".to_string(),
                default: None,
                required: false,
                deprecated: None,
            },
        );
        
        let schema = create_schema_with_plugin(&section_name, properties);
        let analyzer = create_analyzer_with_schema(schema);
        
        // 创建包含未定义配置项的 TOML
        let toml = format!("[{}]\n{} = \"{}\"\n", section_name, undefined_key, value);
        
        let doc = analyzer.parse(&toml).unwrap();
        let diagnostics = analyzer.validate(&doc);
        
        // 属性：应该生成错误诊断
        let has_undefined_error = diagnostics.iter().any(|d| {
            d.severity == Some(DiagnosticSeverity::ERROR)
                && d.code == Some(lsp_types::NumberOrString::String("undefined-property".to_string()))
                && d.message.contains(&undefined_key)
        });
        
        prop_assert!(has_undefined_error, 
            "未定义的配置项 '{}' 应该生成错误诊断", undefined_key);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    /// Feature: spring-lsp, Property 19: 配置项定义验证（未定义的配置节）
    /// 
    /// For any 配置节，如果该配置节不在 Schema 中定义，验证器应该生成错误诊断。
    #[test]
    fn prop_undefined_section_generates_error(
        defined_section in valid_section_name(),
        undefined_section in valid_section_name(),
        key in valid_property_key(),
        value in valid_string_value()
    ) {
        // 确保两个节名不同
        prop_assume!(defined_section != undefined_section);
        
        // 创建只包含 defined_section 的 Schema
        let properties = HashMap::new();
        let schema = create_schema_with_plugin(&defined_section, properties);
        let analyzer = create_analyzer_with_schema(schema);
        
        // 创建包含未定义配置节的 TOML
        let toml = format!("[{}]\n{} = \"{}\"\n", undefined_section, key, value);
        
        let doc = analyzer.parse(&toml).unwrap();
        let diagnostics = analyzer.validate(&doc);
        
        // 属性：应该生成错误诊断
        let has_undefined_error = diagnostics.iter().any(|d| {
            d.severity == Some(DiagnosticSeverity::ERROR)
                && d.code == Some(lsp_types::NumberOrString::String("undefined-section".to_string()))
                && d.message.contains(&undefined_section)
        });
        
        prop_assert!(has_undefined_error, 
            "未定义的配置节 '{}' 应该生成错误诊断", undefined_section);
    }
}

// ============================================================================
// Property 20: 配置类型验证
// **Validates: Requirements 5.2**
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    /// Feature: spring-lsp, Property 20: 配置类型验证
    /// 
    /// For any 配置项的值，如果其类型与 Schema 中定义的类型不匹配，验证器应该生成类型错误诊断。
    #[test]
    fn prop_type_mismatch_generates_error(
        section_name in valid_section_name(),
        key in valid_property_key(),
        string_value in valid_string_value()
    ) {
        // 创建期望整数类型的 Schema
        let mut properties = HashMap::new();
        properties.insert(
            key.clone(),
            PropertySchema {
                name: key.clone(),
                type_info: TypeInfo::Integer {
                    min: None,
                    max: None,
                },
                description: "Test integer property".to_string(),
                default: None,
                required: false,
                deprecated: None,
            },
        );
        
        let schema = create_schema_with_plugin(&section_name, properties);
        let analyzer = create_analyzer_with_schema(schema);
        
        // 创建包含字符串值的 TOML（类型不匹配）
        let toml = format!("[{}]\n{} = \"{}\"\n", section_name, key, string_value);
        
        let doc = analyzer.parse(&toml).unwrap();
        let diagnostics = analyzer.validate(&doc);
        
        // 属性：应该生成类型错误诊断
        let has_type_error = diagnostics.iter().any(|d| {
            d.severity == Some(DiagnosticSeverity::ERROR)
                && d.code == Some(lsp_types::NumberOrString::String("type-mismatch".to_string()))
                && d.message.contains("类型不匹配")
        });
        
        prop_assert!(has_type_error, 
            "类型不匹配应该生成错误诊断（期望整数，实际字符串）");
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    /// Feature: spring-lsp, Property 20: 配置类型验证（整数 vs 字符串）
    /// 
    /// 当 Schema 期望字符串但提供整数时，应该生成类型错误
    #[test]
    fn prop_type_mismatch_integer_for_string(
        section_name in valid_section_name(),
        key in valid_property_key(),
        int_value in integer_value()
    ) {
        // 创建期望字符串类型的 Schema
        let mut properties = HashMap::new();
        properties.insert(
            key.clone(),
            PropertySchema {
                name: key.clone(),
                type_info: TypeInfo::String {
                    enum_values: None,
                    min_length: None,
                    max_length: None,
                },
                description: "Test string property".to_string(),
                default: None,
                required: false,
                deprecated: None,
            },
        );
        
        let schema = create_schema_with_plugin(&section_name, properties);
        let analyzer = create_analyzer_with_schema(schema);
        
        // 创建包含整数值的 TOML（类型不匹配）
        let toml = format!("[{}]\n{} = {}\n", section_name, key, int_value);
        
        let doc = analyzer.parse(&toml).unwrap();
        let diagnostics = analyzer.validate(&doc);
        
        // 属性：应该生成类型错误诊断
        let has_type_error = diagnostics.iter().any(|d| {
            d.severity == Some(DiagnosticSeverity::ERROR)
                && d.code == Some(lsp_types::NumberOrString::String("type-mismatch".to_string()))
        });
        
        prop_assert!(has_type_error, 
            "类型不匹配应该生成错误诊断（期望字符串，实际整数）");
    }
}

// ============================================================================
// Property 21: 必需配置项检查
// **Validates: Requirements 5.3**
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    /// Feature: spring-lsp, Property 21: 必需配置项检查
    /// 
    /// For any 在 Schema 中标记为必需的配置项，如果在配置文件中缺失，验证器应该生成警告诊断。
    #[test]
    fn prop_missing_required_property_generates_warning(
        section_name in valid_section_name(),
        required_key in valid_property_key(),
        optional_key in valid_property_key(),
        value in valid_string_value()
    ) {
        // 确保两个键不同
        prop_assume!(required_key != optional_key);
        
        // 创建包含必需配置项的 Schema
        let mut properties = HashMap::new();
        properties.insert(
            required_key.clone(),
            PropertySchema {
                name: required_key.clone(),
                type_info: TypeInfo::String {
                    enum_values: None,
                    min_length: None,
                    max_length: None,
                },
                description: "Required property".to_string(),
                default: None,
                required: true,  // 标记为必需
                deprecated: None,
            },
        );
        properties.insert(
            optional_key.clone(),
            PropertySchema {
                name: optional_key.clone(),
                type_info: TypeInfo::String {
                    enum_values: None,
                    min_length: None,
                    max_length: None,
                },
                description: "Optional property".to_string(),
                default: None,
                required: false,
                deprecated: None,
            },
        );
        
        let schema = create_schema_with_plugin(&section_name, properties);
        let analyzer = create_analyzer_with_schema(schema);
        
        // 创建只包含可选配置项的 TOML（缺少必需配置项）
        let toml = format!("[{}]\n{} = \"{}\"\n", section_name, optional_key, value);
        
        let doc = analyzer.parse(&toml).unwrap();
        let diagnostics = analyzer.validate(&doc);
        
        // 属性：应该生成警告诊断
        let has_missing_warning = diagnostics.iter().any(|d| {
            d.severity == Some(DiagnosticSeverity::WARNING)
                && d.code == Some(lsp_types::NumberOrString::String("missing-required-property".to_string()))
                && d.message.contains(&required_key)
        });
        
        prop_assert!(has_missing_warning, 
            "缺少必需配置项 '{}' 应该生成警告诊断", required_key);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    /// Feature: spring-lsp, Property 21: 必需配置项检查（提供必需项时无警告）
    /// 
    /// 当所有必需配置项都存在时，不应该生成缺失警告
    #[test]
    fn prop_required_property_present_no_warning(
        section_name in valid_section_name(),
        required_key in valid_property_key(),
        value in valid_string_value()
    ) {
        // 创建包含必需配置项的 Schema
        let mut properties = HashMap::new();
        properties.insert(
            required_key.clone(),
            PropertySchema {
                name: required_key.clone(),
                type_info: TypeInfo::String {
                    enum_values: None,
                    min_length: None,
                    max_length: None,
                },
                description: "Required property".to_string(),
                default: None,
                required: true,
                deprecated: None,
            },
        );
        
        let schema = create_schema_with_plugin(&section_name, properties);
        let analyzer = create_analyzer_with_schema(schema);
        
        // 创建包含必需配置项的 TOML
        let toml = format!("[{}]\n{} = \"{}\"\n", section_name, required_key, value);
        
        let doc = analyzer.parse(&toml).unwrap();
        let diagnostics = analyzer.validate(&doc);
        
        // 属性：不应该有缺失必需配置项的警告
        let has_missing_warning = diagnostics.iter().any(|d| {
            d.code == Some(lsp_types::NumberOrString::String("missing-required-property".to_string()))
        });
        
        prop_assert!(!has_missing_warning, 
            "提供了必需配置项时不应该生成缺失警告");
    }
}

// ============================================================================
// Property 22: 废弃配置项警告
// **Validates: Requirements 5.4**
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    /// Feature: spring-lsp, Property 22: 废弃配置项警告
    /// 
    /// For any 在 Schema 中标记为废弃的配置项，如果在配置文件中使用，验证器应该生成废弃警告并提供替代建议。
    #[test]
    fn prop_deprecated_property_generates_warning(
        section_name in valid_section_name(),
        deprecated_key in valid_property_key(),
        value in valid_string_value(),
        deprecation_msg in "[a-zA-Z ]{10,50}"
    ) {
        // 创建包含废弃配置项的 Schema
        let mut properties = HashMap::new();
        properties.insert(
            deprecated_key.clone(),
            PropertySchema {
                name: deprecated_key.clone(),
                type_info: TypeInfo::String {
                    enum_values: None,
                    min_length: None,
                    max_length: None,
                },
                description: "Deprecated property".to_string(),
                default: None,
                required: false,
                deprecated: Some(deprecation_msg.clone()),  // 标记为废弃
            },
        );
        
        let schema = create_schema_with_plugin(&section_name, properties);
        let analyzer = create_analyzer_with_schema(schema);
        
        // 创建使用废弃配置项的 TOML
        let toml = format!("[{}]\n{} = \"{}\"\n", section_name, deprecated_key, value);
        
        let doc = analyzer.parse(&toml).unwrap();
        let diagnostics = analyzer.validate(&doc);
        
        // 属性：应该生成废弃警告
        let has_deprecated_warning = diagnostics.iter().any(|d| {
            d.severity == Some(DiagnosticSeverity::WARNING)
                && d.code == Some(lsp_types::NumberOrString::String("deprecated-property".to_string()))
                && d.message.contains("已废弃")
                && d.message.contains(&deprecation_msg)
        });
        
        prop_assert!(has_deprecated_warning, 
            "使用废弃配置项 '{}' 应该生成警告诊断", deprecated_key);
    }
}

// ============================================================================
// Property 23: 环境变量语法验证
// **Validates: Requirements 5.5**
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    /// Feature: spring-lsp, Property 23: 环境变量语法验证（空变量名）
    /// 
    /// For any 环境变量插值表达式，如果变量名为空，验证器应该生成语法错误诊断。
    #[test]
    fn prop_empty_env_var_name_generates_error(
        section_name in valid_section_name(),
        key in valid_property_key()
    ) {
        // 创建简单的 Schema
        let mut properties = HashMap::new();
        properties.insert(
            key.clone(),
            PropertySchema {
                name: key.clone(),
                type_info: TypeInfo::String {
                    enum_values: None,
                    min_length: None,
                    max_length: None,
                },
                description: "Test property".to_string(),
                default: None,
                required: false,
                deprecated: None,
            },
        );
        
        let schema = create_schema_with_plugin(&section_name, properties);
        let analyzer = create_analyzer_with_schema(schema);
        
        // 创建包含空环境变量名的 TOML
        let toml = format!("[{}]\n{} = \"${{}}\"\n", section_name, key);
        
        let doc = analyzer.parse(&toml).unwrap();
        let diagnostics = analyzer.validate(&doc);
        
        // 属性：应该生成错误诊断
        let has_empty_var_error = diagnostics.iter().any(|d| {
            d.severity == Some(DiagnosticSeverity::ERROR)
                && d.code == Some(lsp_types::NumberOrString::String("empty-var-name".to_string()))
                && d.message.contains("环境变量名不能为空")
        });
        
        prop_assert!(has_empty_var_error, 
            "空环境变量名应该生成错误诊断");
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    /// Feature: spring-lsp, Property 23: 环境变量语法验证（无效变量名）
    /// 
    /// For any 环境变量插值表达式，如果变量名不符合命名规范，验证器应该生成警告诊断。
    #[test]
    fn prop_invalid_env_var_name_generates_warning(
        section_name in valid_section_name(),
        key in valid_property_key(),
        invalid_var_name in invalid_env_var_name()
    ) {
        // 创建简单的 Schema
        let mut properties = HashMap::new();
        properties.insert(
            key.clone(),
            PropertySchema {
                name: key.clone(),
                type_info: TypeInfo::String {
                    enum_values: None,
                    min_length: None,
                    max_length: None,
                },
                description: "Test property".to_string(),
                default: None,
                required: false,
                deprecated: None,
            },
        );
        
        let schema = create_schema_with_plugin(&section_name, properties);
        let analyzer = create_analyzer_with_schema(schema);
        
        // 创建包含无效环境变量名的 TOML
        let toml = format!("[{}]\n{} = \"${{{}}}\"\n", section_name, key, invalid_var_name);
        
        let doc = analyzer.parse(&toml).unwrap();
        let diagnostics = analyzer.validate(&doc);
        
        // 属性：应该生成警告诊断
        let has_invalid_var_warning = diagnostics.iter().any(|d| {
            d.severity == Some(DiagnosticSeverity::WARNING)
                && d.code == Some(lsp_types::NumberOrString::String("invalid-var-name".to_string()))
                && d.message.contains("不符合命名规范")
        });
        
        prop_assert!(has_invalid_var_warning, 
            "无效环境变量名 '{}' 应该生成警告诊断", invalid_var_name);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    /// Feature: spring-lsp, Property 23: 环境变量语法验证（有效变量名）
    /// 
    /// 当环境变量名符合命名规范时，不应该生成错误或警告
    #[test]
    fn prop_valid_env_var_name_no_error(
        section_name in valid_section_name(),
        key in valid_property_key(),
        valid_var_name in valid_env_var_name()
    ) {
        // 创建简单的 Schema
        let mut properties = HashMap::new();
        properties.insert(
            key.clone(),
            PropertySchema {
                name: key.clone(),
                type_info: TypeInfo::String {
                    enum_values: None,
                    min_length: None,
                    max_length: None,
                },
                description: "Test property".to_string(),
                default: None,
                required: false,
                deprecated: None,
            },
        );
        
        let schema = create_schema_with_plugin(&section_name, properties);
        let analyzer = create_analyzer_with_schema(schema);
        
        // 创建包含有效环境变量名的 TOML
        let toml = format!("[{}]\n{} = \"${{{}}}\"\n", section_name, key, valid_var_name);
        
        let doc = analyzer.parse(&toml).unwrap();
        let diagnostics = analyzer.validate(&doc);
        
        // 属性：不应该有环境变量相关的错误或警告
        let has_env_var_issue = diagnostics.iter().any(|d| {
            d.code == Some(lsp_types::NumberOrString::String("empty-var-name".to_string()))
                || d.code == Some(lsp_types::NumberOrString::String("invalid-var-name".to_string()))
        });
        
        prop_assert!(!has_env_var_issue, 
            "有效环境变量名 '{}' 不应该生成错误或警告", valid_var_name);
    }
}

// ============================================================================
// Property 24: 配置值范围验证
// **Validates: Requirements 5.6**
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    /// Feature: spring-lsp, Property 24: 配置值范围验证（整数最小值）
    /// 
    /// For any 配置项的值，如果小于 Schema 定义的最小值，验证器应该生成范围错误诊断。
    #[test]
    fn prop_integer_below_min_generates_error(
        section_name in valid_section_name(),
        key in valid_property_key(),
        min_value in 1i64..1000i64,
        offset in 1i64..100i64
    ) {
        let below_min = min_value - offset;
        
        // 创建有最小值限制的 Schema
        let mut properties = HashMap::new();
        properties.insert(
            key.clone(),
            PropertySchema {
                name: key.clone(),
                type_info: TypeInfo::Integer {
                    min: Some(min_value),
                    max: None,
                },
                description: "Test integer property".to_string(),
                default: None,
                required: false,
                deprecated: None,
            },
        );
        
        let schema = create_schema_with_plugin(&section_name, properties);
        let analyzer = create_analyzer_with_schema(schema);
        
        // 创建包含小于最小值的 TOML
        let toml = format!("[{}]\n{} = {}\n", section_name, key, below_min);
        
        let doc = analyzer.parse(&toml).unwrap();
        let diagnostics = analyzer.validate(&doc);
        
        // 属性：应该生成范围错误诊断
        let has_range_error = diagnostics.iter().any(|d| {
            d.severity == Some(DiagnosticSeverity::ERROR)
                && d.code == Some(lsp_types::NumberOrString::String("value-too-small".to_string()))
                && d.message.contains("小于最小值")
        });
        
        prop_assert!(has_range_error, 
            "值 {} 小于最小值 {} 应该生成错误诊断", below_min, min_value);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    /// Feature: spring-lsp, Property 24: 配置值范围验证（整数最大值）
    /// 
    /// For any 配置项的值，如果超过 Schema 定义的最大值，验证器应该生成范围错误诊断。
    #[test]
    fn prop_integer_above_max_generates_error(
        section_name in valid_section_name(),
        key in valid_property_key(),
        max_value in 1i64..1000i64,
        offset in 1i64..100i64
    ) {
        let above_max = max_value + offset;
        
        // 创建有最大值限制的 Schema
        let mut properties = HashMap::new();
        properties.insert(
            key.clone(),
            PropertySchema {
                name: key.clone(),
                type_info: TypeInfo::Integer {
                    min: None,
                    max: Some(max_value),
                },
                description: "Test integer property".to_string(),
                default: None,
                required: false,
                deprecated: None,
            },
        );
        
        let schema = create_schema_with_plugin(&section_name, properties);
        let analyzer = create_analyzer_with_schema(schema);
        
        // 创建包含超过最大值的 TOML
        let toml = format!("[{}]\n{} = {}\n", section_name, key, above_max);
        
        let doc = analyzer.parse(&toml).unwrap();
        let diagnostics = analyzer.validate(&doc);
        
        // 属性：应该生成范围错误诊断
        let has_range_error = diagnostics.iter().any(|d| {
            d.severity == Some(DiagnosticSeverity::ERROR)
                && d.code == Some(lsp_types::NumberOrString::String("value-too-large".to_string()))
                && d.message.contains("超过最大值")
        });
        
        prop_assert!(has_range_error, 
            "值 {} 超过最大值 {} 应该生成错误诊断", above_max, max_value);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    /// Feature: spring-lsp, Property 24: 配置值范围验证（整数在范围内）
    /// 
    /// 当整数值在允许范围内时，不应该生成范围错误
    #[test]
    fn prop_integer_within_range_no_error(
        section_name in valid_section_name(),
        key in valid_property_key(),
        min_value in 1i64..100i64,
        max_value in 200i64..1000i64
    ) {
        // 生成范围内的值
        let value = (min_value + max_value) / 2;
        
        // 创建有范围限制的 Schema
        let mut properties = HashMap::new();
        properties.insert(
            key.clone(),
            PropertySchema {
                name: key.clone(),
                type_info: TypeInfo::Integer {
                    min: Some(min_value),
                    max: Some(max_value),
                },
                description: "Test integer property".to_string(),
                default: None,
                required: false,
                deprecated: None,
            },
        );
        
        let schema = create_schema_with_plugin(&section_name, properties);
        let analyzer = create_analyzer_with_schema(schema);
        
        // 创建包含范围内值的 TOML
        let toml = format!("[{}]\n{} = {}\n", section_name, key, value);
        
        let doc = analyzer.parse(&toml).unwrap();
        let diagnostics = analyzer.validate(&doc);
        
        // 属性：不应该有范围错误
        let has_range_error = diagnostics.iter().any(|d| {
            d.code == Some(lsp_types::NumberOrString::String("value-too-small".to_string()))
                || d.code == Some(lsp_types::NumberOrString::String("value-too-large".to_string()))
        });
        
        prop_assert!(!has_range_error, 
            "值 {} 在范围 [{}, {}] 内不应该生成范围错误", value, min_value, max_value);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    /// Feature: spring-lsp, Property 24: 配置值范围验证（字符串长度最小值）
    /// 
    /// For any 字符串配置项，如果长度小于最小长度，验证器应该生成范围错误诊断。
    #[test]
    fn prop_string_below_min_length_generates_error(
        section_name in valid_section_name(),
        key in valid_property_key(),
        min_length in 5usize..20usize
    ) {
        // 生成长度小于最小长度的字符串
        let short_string = "a".repeat(min_length - 1);
        
        // 创建有最小长度限制的 Schema
        let mut properties = HashMap::new();
        properties.insert(
            key.clone(),
            PropertySchema {
                name: key.clone(),
                type_info: TypeInfo::String {
                    enum_values: None,
                    min_length: Some(min_length),
                    max_length: None,
                },
                description: "Test string property".to_string(),
                default: None,
                required: false,
                deprecated: None,
            },
        );
        
        let schema = create_schema_with_plugin(&section_name, properties);
        let analyzer = create_analyzer_with_schema(schema);
        
        // 创建包含过短字符串的 TOML
        let toml = format!("[{}]\n{} = \"{}\"\n", section_name, key, short_string);
        
        let doc = analyzer.parse(&toml).unwrap();
        let diagnostics = analyzer.validate(&doc);
        
        // 属性：应该生成范围错误诊断
        let has_range_error = diagnostics.iter().any(|d| {
            d.severity == Some(DiagnosticSeverity::ERROR)
                && d.code == Some(lsp_types::NumberOrString::String("string-too-short".to_string()))
                && d.message.contains("小于最小长度")
        });
        
        prop_assert!(has_range_error, 
            "字符串长度 {} 小于最小长度 {} 应该生成错误诊断", 
            short_string.len(), min_length);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    /// Feature: spring-lsp, Property 24: 配置值范围验证（字符串长度最大值）
    /// 
    /// For any 字符串配置项，如果长度超过最大长度，验证器应该生成范围错误诊断。
    #[test]
    fn prop_string_above_max_length_generates_error(
        section_name in valid_section_name(),
        key in valid_property_key(),
        max_length in 5usize..20usize
    ) {
        // 生成长度超过最大长度的字符串
        let long_string = "a".repeat(max_length + 1);
        
        // 创建有最大长度限制的 Schema
        let mut properties = HashMap::new();
        properties.insert(
            key.clone(),
            PropertySchema {
                name: key.clone(),
                type_info: TypeInfo::String {
                    enum_values: None,
                    min_length: None,
                    max_length: Some(max_length),
                },
                description: "Test string property".to_string(),
                default: None,
                required: false,
                deprecated: None,
            },
        );
        
        let schema = create_schema_with_plugin(&section_name, properties);
        let analyzer = create_analyzer_with_schema(schema);
        
        // 创建包含过长字符串的 TOML
        let toml = format!("[{}]\n{} = \"{}\"\n", section_name, key, long_string);
        
        let doc = analyzer.parse(&toml).unwrap();
        let diagnostics = analyzer.validate(&doc);
        
        // 属性：应该生成范围错误诊断
        let has_range_error = diagnostics.iter().any(|d| {
            d.severity == Some(DiagnosticSeverity::ERROR)
                && d.code == Some(lsp_types::NumberOrString::String("string-too-long".to_string()))
                && d.message.contains("超过最大长度")
        });
        
        prop_assert!(has_range_error, 
            "字符串长度 {} 超过最大长度 {} 应该生成错误诊断", 
            long_string.len(), max_length);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    /// Feature: spring-lsp, Property 24: 配置值范围验证（枚举值）
    /// 
    /// For any 枚举类型的配置项，如果值不在枚举列表中，验证器应该生成范围错误诊断。
    #[test]
    fn prop_invalid_enum_value_generates_error(
        section_name in valid_section_name(),
        key in valid_property_key(),
        valid_value1 in "[a-z]{3,10}",
        valid_value2 in "[a-z]{3,10}",
        invalid_value in "[A-Z]{3,10}"
    ) {
        // 确保无效值不在枚举列表中
        prop_assume!(invalid_value.to_lowercase() != valid_value1);
        prop_assume!(invalid_value.to_lowercase() != valid_value2);
        
        // 创建有枚举值限制的 Schema
        let mut properties = HashMap::new();
        properties.insert(
            key.clone(),
            PropertySchema {
                name: key.clone(),
                type_info: TypeInfo::String {
                    enum_values: Some(vec![valid_value1.clone(), valid_value2.clone()]),
                    min_length: None,
                    max_length: None,
                },
                description: "Test enum property".to_string(),
                default: None,
                required: false,
                deprecated: None,
            },
        );
        
        let schema = create_schema_with_plugin(&section_name, properties);
        let analyzer = create_analyzer_with_schema(schema);
        
        // 创建包含无效枚举值的 TOML
        let toml = format!("[{}]\n{} = \"{}\"\n", section_name, key, invalid_value);
        
        let doc = analyzer.parse(&toml).unwrap();
        let diagnostics = analyzer.validate(&doc);
        
        // 属性：应该生成范围错误诊断
        let has_enum_error = diagnostics.iter().any(|d| {
            d.severity == Some(DiagnosticSeverity::ERROR)
                && d.code == Some(lsp_types::NumberOrString::String("invalid-enum-value".to_string()))
                && d.message.contains("不在允许的枚举值中")
        });
        
        prop_assert!(has_enum_error, 
            "无效枚举值 '{}' 应该生成错误诊断", invalid_value);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    /// Feature: spring-lsp, Property 24: 配置值范围验证（有效枚举值）
    /// 
    /// 当枚举值在允许列表中时，不应该生成范围错误
    #[test]
    fn prop_valid_enum_value_no_error(
        section_name in valid_section_name(),
        key in valid_property_key(),
        valid_values in prop::collection::vec("[a-z]{3,10}", 2..5)
    ) {
        // 选择一个有效值
        let selected_value = &valid_values[0];
        
        // 创建有枚举值限制的 Schema
        let mut properties = HashMap::new();
        properties.insert(
            key.clone(),
            PropertySchema {
                name: key.clone(),
                type_info: TypeInfo::String {
                    enum_values: Some(valid_values.clone()),
                    min_length: None,
                    max_length: None,
                },
                description: "Test enum property".to_string(),
                default: None,
                required: false,
                deprecated: None,
            },
        );
        
        let schema = create_schema_with_plugin(&section_name, properties);
        let analyzer = create_analyzer_with_schema(schema);
        
        // 创建包含有效枚举值的 TOML
        let toml = format!("[{}]\n{} = \"{}\"\n", section_name, key, selected_value);
        
        let doc = analyzer.parse(&toml).unwrap();
        let diagnostics = analyzer.validate(&doc);
        
        // 属性：不应该有枚举值错误
        let has_enum_error = diagnostics.iter().any(|d| {
            d.code == Some(lsp_types::NumberOrString::String("invalid-enum-value".to_string()))
        });
        
        prop_assert!(!has_enum_error, 
            "有效枚举值 '{}' 不应该生成错误诊断", selected_value);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    /// Feature: spring-lsp, Property 24: 配置值范围验证（浮点数范围）
    /// 
    /// For any 浮点数配置项，如果值超出范围，验证器应该生成范围错误诊断。
    #[test]
    fn prop_float_out_of_range_generates_error(
        section_name in valid_section_name(),
        key in valid_property_key(),
        min_value in 0.0f64..100.0f64,
        max_value in 200.0f64..1000.0f64
    ) {
        // 生成超出范围的值（小于最小值）
        let below_min = min_value - 10.0;
        
        // 创建有范围限制的 Schema
        let mut properties = HashMap::new();
        properties.insert(
            key.clone(),
            PropertySchema {
                name: key.clone(),
                type_info: TypeInfo::Float {
                    min: Some(min_value),
                    max: Some(max_value),
                },
                description: "Test float property".to_string(),
                default: None,
                required: false,
                deprecated: None,
            },
        );
        
        let schema = create_schema_with_plugin(&section_name, properties);
        let analyzer = create_analyzer_with_schema(schema);
        
        // 创建包含超出范围值的 TOML
        let toml = format!("[{}]\n{} = {}\n", section_name, key, below_min);
        
        let doc = analyzer.parse(&toml).unwrap();
        let diagnostics = analyzer.validate(&doc);
        
        // 属性：应该生成范围错误诊断
        let has_range_error = diagnostics.iter().any(|d| {
            d.severity == Some(DiagnosticSeverity::ERROR)
                && d.code == Some(lsp_types::NumberOrString::String("value-too-small".to_string()))
        });
        
        prop_assert!(has_range_error, 
            "浮点数值 {} 小于最小值 {} 应该生成错误诊断", below_min, min_value);
    }
}
