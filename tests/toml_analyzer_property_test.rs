//! TomlAnalyzer 属性测试
//!
//! 使用 proptest 验证 TOML 解析器在随机生成的输入下的正确性

use proptest::prelude::*;
use spring_lsp::schema::SchemaProvider;
use spring_lsp::toml_analyzer::TomlAnalyzer;

fn create_analyzer() -> TomlAnalyzer {
    let schema_provider = SchemaProvider::default();
    TomlAnalyzer::new(schema_provider)
}

// ============================================================================
// 测试策略：生成有效的 TOML 内容
// ============================================================================

/// 生成有效的 TOML 配置节名称
fn valid_section_name() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_-]{0,20}"
}

/// 生成有效的 TOML 属性键
fn valid_property_key() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,20}"
}

/// 生成有效的字符串值（不包含特殊字符）
fn valid_string_value() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9_./:-]{0,50}"
}

/// 生成有效的环境变量名
fn valid_env_var_name() -> impl Strategy<Value = String> {
    "[A-Z][A-Z0-9_]{0,20}"
}

/// 生成有效的 TOML 配置节
fn valid_toml_section() -> impl Strategy<Value = String> {
    (
        valid_section_name(),
        prop::collection::vec((valid_property_key(), valid_string_value()), 0..5),
    )
        .prop_map(|(section, properties)| {
            let mut toml = format!("[{}]\n", section);
            for (key, value) in properties {
                toml.push_str(&format!("{} = \"{}\"\n", key, value));
            }
            toml
        })
}

/// 生成包含环境变量的 TOML 配置
fn toml_with_env_var() -> impl Strategy<Value = (String, String, Option<String>)> {
    (
        valid_section_name(),
        valid_env_var_name(),
        prop::option::of(valid_string_value()),
    )
        .prop_map(|(section, var_name, default)| {
            let value = if let Some(def) = &default {
                format!("${{{}:{}}}", var_name, def)
            } else {
                format!("${{{}}}", var_name)
            };
            let toml = format!("[{}]\nhost = \"{}\"\n", section, value);
            (toml, var_name, default)
        })
}

/// 生成多环境配置文件名
fn multi_env_config_name() -> impl Strategy<Value = String> {
    prop::sample::select(vec![
        "app.toml".to_string(),
        "app-dev.toml".to_string(),
        "app-test.toml".to_string(),
        "app-prod.toml".to_string(),
        "app-staging.toml".to_string(),
    ])
}

/// 生成无效的 TOML 内容
fn invalid_toml() -> impl Strategy<Value = String> {
    prop::sample::select(vec![
        "[invalid".to_string(),               // 未闭合的节
        "[section\nkey = ".to_string(),       // 未闭合的节和未完成的赋值
        "key = \"unclosed".to_string(),       // 未闭合的字符串
        "[section]\nkey = value".to_string(), // 未引用的字符串值
        "[section]\nkey =".to_string(),       // 缺少值
        "= value".to_string(),                // 缺少键
    ])
}

// ============================================================================
// Property 6: TOML 解析成功性
// **Validates: Requirements 2.1**
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: spring-lsp, Property 6: TOML 解析成功性
    ///
    /// For any 语法正确的 TOML 文件，解析器应该成功解析并返回文档结构。
    #[test]
    fn prop_parse_valid_toml_succeeds(
        sections in prop::collection::vec(valid_toml_section(), 0..10)
    ) {
        let analyzer = create_analyzer();
        let toml = sections.join("\n");

        let result = analyzer.parse(&toml);

        // 属性：解析应该成功
        prop_assert!(result.is_ok(), "解析有效的 TOML 应该成功，但失败了: {:?}", result.err());

        let doc = result.unwrap();

        // 属性：文档应该包含配置节
        // 注意：空的 TOML 也是有效的，所以配置节数量可以是 0
        prop_assert!(doc.config_sections.len() <= sections.len(),
            "配置节数量不应超过输入的节数量");
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: spring-lsp, Property 6: TOML 解析成功性（空文件）
    ///
    /// 空的 TOML 文件应该成功解析
    #[test]
    fn prop_parse_empty_toml_succeeds(_dummy in 0..100u32) {
        let analyzer = create_analyzer();
        let result = analyzer.parse("");

        // 属性：空文件应该成功解析
        prop_assert!(result.is_ok(), "解析空 TOML 应该成功");

        let doc = result.unwrap();

        // 属性：空文件应该没有配置节
        prop_assert_eq!(doc.config_sections.len(), 0);
        prop_assert_eq!(doc.env_vars.len(), 0);
    }
}

// ============================================================================
// Property 7: TOML 错误报告
// **Validates: Requirements 2.2**
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: spring-lsp, Property 7: TOML 错误报告
    ///
    /// For any 包含语法错误的 TOML 文件，解析器应该返回包含错误位置和描述的诊断信息。
    #[test]
    fn prop_parse_invalid_toml_returns_error(
        invalid in invalid_toml()
    ) {
        let analyzer = create_analyzer();
        let result = analyzer.parse(&invalid);

        // 属性：解析应该失败
        prop_assert!(result.is_err(), "解析无效的 TOML 应该失败");

        let error = result.unwrap_err();

        // 属性：错误消息应该包含 "TOML 语法错误"
        prop_assert!(error.contains("TOML 语法错误"),
            "错误消息应该包含 'TOML 语法错误'，实际: {}", error);

        // 属性：错误消息应该非空
        prop_assert!(!error.is_empty(), "错误消息不应为空");
    }
}

// ============================================================================
// Property 8: 环境变量识别
// **Validates: Requirements 2.3**
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: spring-lsp, Property 8: 环境变量识别
    ///
    /// For any 包含 `${VAR:default}` 格式的 TOML 文件，解析器应该正确提取变量名 `VAR` 和默认值 `default`。
    #[test]
    fn prop_env_var_extraction_with_default(
        (toml, expected_name, expected_default) in toml_with_env_var()
    ) {
        let analyzer = create_analyzer();
        let result = analyzer.parse(&toml);

        // 属性：解析应该成功
        prop_assert!(result.is_ok(), "解析包含环境变量的 TOML 应该成功");

        let doc = result.unwrap();

        // 属性：应该识别到环境变量
        prop_assert_eq!(doc.env_vars.len(), 1, "应该识别到 1 个环境变量");

        let env_var = &doc.env_vars[0];

        // 属性：变量名应该正确
        prop_assert_eq!(&env_var.name, &expected_name,
            "环境变量名应该是 {}，实际: {}", expected_name, env_var.name);

        // 属性：默认值应该正确
        prop_assert_eq!(&env_var.default, &expected_default,
            "默认值应该是 {:?}，实际: {:?}", expected_default, env_var.default);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: spring-lsp, Property 8: 环境变量识别（多个变量）
    ///
    /// 应该能够识别多个环境变量
    #[test]
    fn prop_multiple_env_vars_extraction(
        var_count in 1..5usize,
        var_names in prop::collection::vec(valid_env_var_name(), 1..5)
    ) {
        let var_names = var_names.into_iter().take(var_count).collect::<Vec<_>>();

        // 构建包含多个环境变量的 TOML
        let mut toml = "[config]\n".to_string();
        for (i, var_name) in var_names.iter().enumerate() {
            toml.push_str(&format!("var{} = \"${{{}}}\"\n", i, var_name));
        }

        let analyzer = create_analyzer();
        let result = analyzer.parse(&toml);

        prop_assert!(result.is_ok(), "解析应该成功");

        let doc = result.unwrap();

        // 属性：应该识别到所有环境变量
        prop_assert_eq!(doc.env_vars.len(), var_names.len(),
            "应该识别到 {} 个环境变量，实际: {}", var_names.len(), doc.env_vars.len());

        // 属性：所有变量名都应该被识别
        let extracted_names: Vec<String> = doc.env_vars.iter()
            .map(|v| v.name.clone())
            .collect();

        for expected_name in &var_names {
            prop_assert!(extracted_names.contains(expected_name),
                "应该识别到环境变量 {}，实际识别到: {:?}", expected_name, extracted_names);
        }
    }
}

// ============================================================================
// Property 9: 多环境配置支持
// **Validates: Requirements 2.4**
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: spring-lsp, Property 9: 多环境配置支持
    ///
    /// For any 环境配置文件（如 `app-dev.toml`、`app-prod.toml`），解析器应该能够成功解析。
    #[test]
    fn prop_multi_env_config_support(
        _config_name in multi_env_config_name(),
        sections in prop::collection::vec(valid_toml_section(), 1..5)
    ) {
        let analyzer = create_analyzer();
        let toml = sections.join("\n");

        // 属性：不同环境的配置文件都应该能够解析
        // 注意：文件名本身不影响解析，这里主要测试内容的解析
        let result = analyzer.parse(&toml);

        prop_assert!(result.is_ok(),
            "多环境配置文件应该能够成功解析，但失败了: {:?}", result.err());

        let doc = result.unwrap();

        // 属性：应该能够提取配置节
        prop_assert!(doc.config_sections.len() > 0,
            "多环境配置文件应该包含配置节");
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: spring-lsp, Property 9: 多环境配置支持（不同环境相同结构）
    ///
    /// 不同环境的配置文件应该能够使用相同的配置结构
    #[test]
    fn prop_same_structure_across_envs(
        section_name in valid_section_name(),
        key in valid_property_key(),
        dev_value in valid_string_value(),
        prod_value in valid_string_value()
    ) {
        let analyzer = create_analyzer();

        // 开发环境配置
        let dev_toml = format!("[{}]\n{} = \"{}\"\n", section_name, key, dev_value);
        let dev_result = analyzer.parse(&dev_toml);

        // 生产环境配置
        let prod_toml = format!("[{}]\n{} = \"{}\"\n", section_name, key, prod_value);
        let prod_result = analyzer.parse(&prod_toml);

        // 属性：两个环境的配置都应该成功解析
        prop_assert!(dev_result.is_ok(), "开发环境配置应该成功解析");
        prop_assert!(prod_result.is_ok(), "生产环境配置应该成功解析");

        let dev_doc = dev_result.unwrap();
        let prod_doc = prod_result.unwrap();

        // 属性：两个环境应该有相同的配置节结构
        prop_assert_eq!(dev_doc.config_sections.len(), prod_doc.config_sections.len(),
            "不同环境应该有相同数量的配置节");

        // 属性：配置节名称应该相同
        prop_assert!(dev_doc.config_sections.contains_key(&section_name),
            "开发环境应该包含配置节 {}", section_name);
        prop_assert!(prod_doc.config_sections.contains_key(&section_name),
            "生产环境应该包含配置节 {}", section_name);
    }
}

// ============================================================================
// 额外的属性测试：解析幂等性
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// 解析幂等性：多次解析相同的 TOML 应该产生相同的结果
    #[test]
    fn prop_parse_idempotent(
        sections in prop::collection::vec(valid_toml_section(), 0..5)
    ) {
        let analyzer = create_analyzer();
        let toml = sections.join("\n");

        let result1 = analyzer.parse(&toml);
        let result2 = analyzer.parse(&toml);

        // 属性：两次解析的结果应该一致
        match (result1, result2) {
            (Ok(doc1), Ok(doc2)) => {
                prop_assert_eq!(doc1.config_sections.len(), doc2.config_sections.len(),
                    "两次解析应该产生相同数量的配置节");
                prop_assert_eq!(doc1.env_vars.len(), doc2.env_vars.len(),
                    "两次解析应该产生相同数量的环境变量");
            }
            (Err(_), Err(_)) => {
                // 两次都失败也是一致的
            }
            _ => {
                prop_assert!(false, "两次解析的结果应该一致（都成功或都失败）");
            }
        }
    }
}

// ============================================================================
// 额外的属性测试：配置节数量
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// 配置节数量：解析后的配置节数量应该合理
    #[test]
    fn prop_section_count_reasonable(
        section_count in 1..10usize,
        section_names in prop::collection::hash_set(valid_section_name(), 1..10)
    ) {
        let section_names: Vec<String> = section_names.into_iter().take(section_count).collect();

        // 构建 TOML
        let mut toml = String::new();
        for name in &section_names {
            toml.push_str(&format!("[{}]\n", name));
            toml.push_str("key = \"value\"\n\n");
        }

        let analyzer = create_analyzer();
        let result = analyzer.parse(&toml);

        prop_assert!(result.is_ok(), "解析应该成功");

        let doc = result.unwrap();

        // 属性：配置节数量应该等于输入的节数量
        prop_assert_eq!(doc.config_sections.len(), section_names.len(),
            "配置节数量应该是 {}，实际: {}", section_names.len(), doc.config_sections.len());

        // 属性：所有节名称都应该被识别
        for name in &section_names {
            prop_assert!(doc.config_sections.contains_key(name),
                "应该包含配置节 {}", name);
        }
    }
}
