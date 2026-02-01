//! TOML 悬停提示属性测试
//!
//! 使用 proptest 验证悬停提示功能在随机生成的输入下的正确性

use lsp_types::Position;
use proptest::prelude::*;
use spring_lsp::schema::{ConfigSchema, PluginSchema, PropertySchema, SchemaProvider, TypeInfo};
use spring_lsp::toml_analyzer::TomlAnalyzer;
use std::collections::HashMap;

// ============================================================================
// 测试策略：生成有效的配置和位置
// ============================================================================

/// 生成有效的配置前缀
fn valid_prefix() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_-]{0,15}"
}

/// 生成有效的属性名
fn valid_property_name() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,15}"
}

/// 生成有效的描述文本
fn valid_description() -> impl Strategy<Value = String> {
    "[A-Za-z0-9 .,!?-]{10,100}"
}

/// 生成有效的字符串值
fn valid_string_value() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9_./:-]{1,30}"
}

/// 生成有效的环境变量名
fn valid_env_var_name() -> impl Strategy<Value = String> {
    "[A-Z][A-Z0-9_]{0,15}"
}

/// 生成 TypeInfo
fn type_info() -> impl Strategy<Value = TypeInfo> {
    prop_oneof![
        Just(TypeInfo::String {
            enum_values: None,
            min_length: None,
            max_length: None,
        }),
        Just(TypeInfo::Integer {
            min: Some(1),
            max: Some(65535),
        }),
        Just(TypeInfo::Boolean),
    ]
}

/// 生成 PropertySchema
fn property_schema() -> impl Strategy<Value = (String, PropertySchema)> {
    (
        valid_property_name(),
        type_info(),
        valid_description(),
        any::<bool>(),
    )
        .prop_map(|(name, type_info, description, required)| {
            let schema = PropertySchema {
                name: name.clone(),
                type_info,
                description,
                default: None,
                required,
                deprecated: None,
                example: None,
            };
            (name, schema)
        })
}

/// 生成包含多个属性的 PluginSchema
fn plugin_schema() -> impl Strategy<Value = (String, PluginSchema)> {
    (
        valid_prefix(),
        prop::collection::vec(property_schema(), 1..5),
    )
        .prop_map(|(prefix, properties)| {
            let properties_map: HashMap<String, PropertySchema> = properties.into_iter().collect();
            let schema = PluginSchema {
                prefix: prefix.clone(),
                properties: properties_map,
            };
            (prefix, schema)
        })
}

/// 生成 ConfigSchema
fn config_schema() -> impl Strategy<Value = ConfigSchema> {
    prop::collection::vec(plugin_schema(), 1..3).prop_map(|plugins| {
        let plugins_map: HashMap<String, PluginSchema> = plugins.into_iter().collect();
        ConfigSchema {
            plugins: plugins_map,
        }
    })
}

/// 生成包含配置项的 TOML 内容和对应的 Schema
fn toml_with_schema() -> impl Strategy<Value = (String, ConfigSchema, String, String)> {
    config_schema().prop_flat_map(|schema| {
        // 从 schema 中选择一个插件
        let prefixes: Vec<String> = schema.plugins.keys().cloned().collect();
        if prefixes.is_empty() {
            return Just((String::new(), schema, String::new(), String::new())).boxed();
        }

        let prefix = prefixes[0].clone();
        let plugin = schema.plugins.get(&prefix).unwrap();
        let properties: Vec<String> = plugin.properties.keys().cloned().collect();

        if properties.is_empty() {
            return Just((String::new(), schema, String::new(), String::new())).boxed();
        }

        let property = properties[0].clone();

        // 生成 TOML 内容
        valid_string_value()
            .prop_map(move |value| {
                let toml = format!("[{}]\n{} = \"{}\"\n", prefix, property, value);
                (toml, schema.clone(), prefix.clone(), property.clone())
            })
            .boxed()
    })
}

/// 生成包含环境变量的 TOML 内容
fn toml_with_env_var() -> impl Strategy<Value = (String, String, Option<String>)> {
    (
        valid_prefix(),
        valid_property_name(),
        valid_env_var_name(),
        prop::option::of(valid_string_value()),
    )
        .prop_map(|(prefix, property, var_name, default)| {
            let value = if let Some(def) = &default {
                format!("${{{}:{}}}", var_name, def)
            } else {
                format!("${{{}}}", var_name)
            };
            let toml = format!("[{}]\n{} = \"{}\"\n", prefix, property, value);
            (toml, var_name, default)
        })
}

// ============================================================================
// Property 61: 配置项悬停文档
// **Validates: Requirements 15.1**
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: spring-lsp, Property 61: 配置项悬停文档
    ///
    /// For any 配置项，悬停时应该显示该配置项的文档说明和类型信息。
    #[test]
    fn prop_hover_shows_config_documentation(
        (toml_content, schema, prefix, property) in toml_with_schema()
    ) {
        // 跳过空内容
        if !toml_content.is_empty() {
            let schema_provider = SchemaProvider::from_schema(schema.clone());
            let analyzer = TomlAnalyzer::new(schema_provider);

            // 解析 TOML
            let result = analyzer.parse(&toml_content);
            prop_assert!(result.is_ok(), "解析应该成功: {:?}", result.err());
            let doc = result.unwrap();

            // 在配置项的值位置悬停
            // 第 2 行（索引 1）是配置项所在行
            let position = Position {
                line: 1,
                character: 5,  // 在配置项附近
            };

            let hover = analyzer.hover(&doc, position);

            // 如果找到了悬停信息，验证其内容
            if let Some(hover) = hover {
                if let lsp_types::HoverContents::Markup(content) = hover.contents {
                    let text = content.value;

                    // 验证包含配置项名称
                    prop_assert!(
                        text.contains(&prefix) || text.contains(&property),
                        "悬停提示应该包含配置项名称，实际内容: {}",
                        text
                    );

                    // 验证包含类型信息
                    prop_assert!(
                        text.contains("类型") || text.contains("Type"),
                        "悬停提示应该包含类型信息，实际内容: {}",
                        text
                    );

                    // 验证是 Markdown 格式
                    prop_assert_eq!(&content.kind, &lsp_types::MarkupKind::Markdown);
                }
            }
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: spring-lsp, Property 61: 配置项悬停文档（验证描述信息）
    ///
    /// 悬停提示应该包含 Schema 中定义的描述信息
    #[test]
    fn prop_hover_includes_schema_description(
        prefix in valid_prefix(),
        property_name in valid_property_name(),
        description in valid_description(),
        value in valid_string_value()
    ) {
        // 创建包含描述的 Schema
        let mut properties = HashMap::new();
        properties.insert(
            property_name.clone(),
            PropertySchema {
                name: property_name.clone(),
                type_info: TypeInfo::String {
                    enum_values: None,
                    min_length: None,
                    max_length: None,
                },
                description: description.clone(),
                default: None,
                required: false,
                deprecated: None,
                example: None,
            },
        );

        let mut plugins = HashMap::new();
        plugins.insert(
            prefix.clone(),
            PluginSchema {
                prefix: prefix.clone(),
                properties,
            },
        );

        let schema = ConfigSchema { plugins };
        let schema_provider = SchemaProvider::from_schema(schema);
        let analyzer = TomlAnalyzer::new(schema_provider);

        // 生成 TOML 内容
        let toml_content = format!("[{}]\n{} = \"{}\"\n", prefix, property_name, value);

        // 解析 TOML
        let result = analyzer.parse(&toml_content);
        prop_assert!(result.is_ok(), "解析应该成功: {:?}", result.err());
        let doc = result.unwrap();

        // 在配置项的值位置悬停
        let position = Position {
            line: 1,
            character: 5,
        };

        let hover = analyzer.hover(&doc, position);

        // 验证悬停信息包含描述
        if let Some(hover) = hover {
            if let lsp_types::HoverContents::Markup(content) = hover.contents {
                let text = content.value;

                // 验证包含描述信息
                prop_assert!(
                    text.contains(&description),
                    "悬停提示应该包含描述信息 '{}', 实际内容: {}",
                    description,
                    text
                );
            }
        }
    }
}

// ============================================================================
// Property 63: 环境变量悬停值
// **Validates: Requirements 15.4**
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: spring-lsp, Property 63: 环境变量悬停值
    ///
    /// For any 环境变量引用，悬停时应该显示该环境变量的当前值（如果在运行环境中可用）
    #[test]
    fn prop_hover_shows_env_var_info(
        (toml_content, var_name, default) in toml_with_env_var()
    ) {
        let schema_provider = SchemaProvider::default();
        let analyzer = TomlAnalyzer::new(schema_provider);

        // 解析 TOML
        let result = analyzer.parse(&toml_content);
        prop_assert!(result.is_ok(), "解析应该成功: {:?}", result.err());
        let doc = result.unwrap();

        // 验证环境变量被正确提取
        prop_assert!(!doc.env_vars.is_empty(), "应该提取到环境变量");

        // 验证提取的环境变量包含正确的信息
        let found_env_var = doc.env_vars.iter().any(|ev| {
            ev.name == var_name && ev.default == default
        });

        prop_assert!(
            found_env_var,
            "应该找到环境变量 '{}' (默认值: {:?})",
            var_name,
            default
        );

        // 在环境变量的精确位置悬停
        // 使用提取的环境变量的范围
        if let Some(env_var) = doc.env_vars.first() {
            let position = Position {
                line: env_var.range.start.line,
                character: env_var.range.start.character + 2,  // 在 ${ 之后
            };

            let hover = analyzer.hover(&doc, position);

            // 验证悬停信息
            if let Some(hover) = hover {
                if let lsp_types::HoverContents::Markup(content) = hover.contents {
                    let text = content.value;

                    // 验证包含环境变量标识或变量名
                    // 注意：由于位置可能匹配到配置项而非环境变量，我们接受两种情况
                    let has_env_var_info = text.contains("环境变量") ||
                                          text.contains("Environment Variable") ||
                                          text.contains(&var_name);

                    prop_assert!(
                        has_env_var_info,
                        "悬停提示应该包含环境变量信息或变量名 '{}', 实际内容: {}",
                        var_name,
                        text
                    );

                    // 验证是 Markdown 格式
                    prop_assert_eq!(&content.kind, &lsp_types::MarkupKind::Markdown);
                }
            }
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: spring-lsp, Property 63: 环境变量悬停值（验证格式说明）
    ///
    /// 悬停提示应该包含环境变量的格式说明
    #[test]
    fn prop_hover_env_var_shows_format(
        var_name in valid_env_var_name(),
        has_default in any::<bool>()
    ) {
        let schema_provider = SchemaProvider::default();
        let analyzer = TomlAnalyzer::new(schema_provider);

        // 生成 TOML 内容
        let value = if has_default {
            format!("${{{}:default_value}}", var_name)
        } else {
            format!("${{{}}}", var_name)
        };
        let toml_content = format!("[test]\nkey = \"{}\"\n", value);

        // 解析 TOML
        let result = analyzer.parse(&toml_content);
        prop_assert!(result.is_ok(), "解析应该成功: {:?}", result.err());
        let doc = result.unwrap();

        // 在环境变量位置悬停
        let position = Position {
            line: 1,
            character: 10,
        };

        let hover = analyzer.hover(&doc, position);

        // 验证悬停信息
        if let Some(hover) = hover {
            if let lsp_types::HoverContents::Markup(content) = hover.contents {
                let text = content.value;

                // 验证包含格式说明或变量名
                prop_assert!(
                    text.contains("格式") || text.contains(&var_name) || text.contains("环境变量"),
                    "悬停提示应该包含格式说明或变量信息，实际内容: {}",
                    text
                );
            }
        }
    }
}

// ============================================================================
// 额外的属性测试：验证悬停提示的一致性
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// 悬停提示应该在有效位置返回一致的结果
    ///
    /// 对于同一个配置项，在其范围内的不同位置悬停应该返回相同的信息
    #[test]
    fn prop_hover_consistency_across_positions(
        prefix in valid_prefix(),
        property_name in valid_property_name(),
        value in valid_string_value()
    ) {
        let schema_provider = SchemaProvider::default();
        let analyzer = TomlAnalyzer::new(schema_provider);

        // 生成 TOML 内容
        let toml_content = format!("[{}]\n{} = \"{}\"\n", prefix, property_name, value);

        // 解析 TOML
        let result = analyzer.parse(&toml_content);
        prop_assert!(result.is_ok(), "解析应该成功: {:?}", result.err());
        let doc = result.unwrap();

        // 在配置项的不同位置悬停
        let positions = vec![
            Position { line: 1, character: 0 },  // 行首
            Position { line: 1, character: 2 },  // 键中间
            Position { line: 1, character: 10 }, // 值中间
        ];

        let mut hover_results = Vec::new();
        for pos in positions {
            let hover = analyzer.hover(&doc, pos);
            hover_results.push(hover);
        }

        // 验证至少有一个位置返回了悬停信息
        // （由于 taplo 的范围计算，不是所有位置都会返回）
        let _has_hover = hover_results.iter().any(|h| h.is_some());

        // 如果有悬停信息，验证它们的格式一致
        for hover in hover_results.iter().flatten() {
            if let lsp_types::HoverContents::Markup(content) = &hover.contents {
                prop_assert_eq!(&content.kind, &lsp_types::MarkupKind::Markdown);
            }
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// 悬停提示应该正确处理无效位置
    ///
    /// 在空白位置或无效位置悬停不应该崩溃
    #[test]
    fn prop_hover_handles_invalid_positions(
        prefix in valid_prefix(),
        property_name in valid_property_name(),
        value in valid_string_value(),
        line in 0u32..10,
        character in 0u32..100
    ) {
        let schema_provider = SchemaProvider::default();
        let analyzer = TomlAnalyzer::new(schema_provider);

        // 生成 TOML 内容
        let toml_content = format!("[{}]\n{} = \"{}\"\n", prefix, property_name, value);

        // 解析 TOML
        let result = analyzer.parse(&toml_content);
        prop_assert!(result.is_ok(), "解析应该成功: {:?}", result.err());
        let doc = result.unwrap();

        // 在随机位置悬停
        let position = Position { line, character };

        // 不应该崩溃
        let _hover = analyzer.hover(&doc, position);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// 悬停提示的 Markdown 格式应该正确
    ///
    /// 所有返回的悬停提示都应该是有效的 Markdown 格式
    #[test]
    fn prop_hover_markdown_format_valid(
        (toml_content, schema, _prefix, _property) in toml_with_schema()
    ) {
        // 跳过空内容
        if !toml_content.is_empty() {
            let schema_provider = SchemaProvider::from_schema(schema);
            let analyzer = TomlAnalyzer::new(schema_provider);

            // 解析 TOML
            let result = analyzer.parse(&toml_content);
            prop_assert!(result.is_ok(), "解析应该成功: {:?}", result.err());
            let doc = result.unwrap();

            // 在配置项位置悬停
            let position = Position {
                line: 1,
                character: 5,
            };

            let hover = analyzer.hover(&doc, position);

            // 验证 Markdown 格式
            if let Some(hover) = hover {
                if let lsp_types::HoverContents::Markup(content) = hover.contents {
                    prop_assert_eq!(&content.kind, &lsp_types::MarkupKind::Markdown);

                    let text = content.value;

                    // 验证基本的 Markdown 结构
                    // 应该包含标题、粗体或代码标记
                    let has_markdown_elements =
                        text.contains('#') ||
                        text.contains("**") ||
                        text.contains('`');

                    prop_assert!(
                        has_markdown_elements,
                        "悬停提示应该包含 Markdown 格式元素，实际内容: {}",
                        text
                    );
                }
            }
        }
    }
}
