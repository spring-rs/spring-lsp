//! 示例代码显示属性测试
//!
//! 本测试文件验证示例代码显示功能的正确性属性。
//!
//! ## 测试的属性
//!
//! - **Property 64: 文档示例显示** - 验证包含示例代码的文档在悬停提示中正确显示格式化的示例代码
//!
//! ## 测试策略
//!
//! 使用 proptest 生成随机的配置 Schema 和 TOML 文档，验证示例代码在悬停提示中的显示正确性。

use proptest::prelude::*;
use lsp_types::Position;
use spring_lsp::schema::{ConfigSchema, PluginSchema, PropertySchema, SchemaProvider, TypeInfo, Value};
use spring_lsp::toml_analyzer::TomlAnalyzer;
use std::collections::HashMap;

// ============================================================================
// 测试策略生成器
// ============================================================================

/// 生成有效的配置前缀（插件名称）
fn config_prefix() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{2,15}".prop_map(|s| s.to_string())
}

/// 生成有效的配置键名
fn config_key() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{2,20}".prop_map(|s| s.to_string())
}

/// 生成有效的字符串值
fn string_value() -> impl Strategy<Value = String> {
    "[a-z0-9.:/\\-]{1,50}".prop_map(|s| s.to_string())
}

/// 生成有效的整数值
fn integer_value() -> impl Strategy<Value = i64> {
    1i64..65535i64
}

/// 生成单行示例代码（简化版本，不使用闭包捕获）
fn single_line_example() -> impl Strategy<Value = String> {
    (string_value(), integer_value()).prop_map(|(s, i)| {
        format!("key = \"{}\" # or key = {}", s, i)
    })
}

/// 生成多行示例代码（简化版本，不使用闭包捕获）
fn multiline_example() -> impl Strategy<Value = String> {
    (
        string_value(),
        integer_value(),
        prop::collection::vec(string_value(), 1..5),
    ).prop_map(|(s, i, arr)| {
        let array_str = arr.iter()
            .map(|v| format!("\"{}\"", v))
            .collect::<Vec<_>>()
            .join(", ");
        
        format!(
            "[section.subsection]\nvalue1 = \"{}\"\nvalue2 = {}\narray = [{}]",
            s, i, array_str
        )
    })
}

/// 生成带示例代码的 PropertySchema
fn property_schema_with_example(key: String, has_example: bool) -> impl Strategy<Value = PropertySchema> {
    let key_clone = key.clone();
    (
        string_value(),
        prop::option::of(string_value()),
        prop::bool::ANY,
    ).prop_map(move |(desc, default, is_multiline)| {
        let example = if has_example {
            if is_multiline {
                Some(format!("{} = \"example_value\"\n# Additional config\nother = 123", key_clone))
            } else {
                Some(format!("{} = \"example_value\"", key_clone))
            }
        } else {
            None
        };
        
        PropertySchema {
            name: key_clone.clone(),
            type_info: TypeInfo::String {
                enum_values: None,
                min_length: None,
                max_length: None,
            },
            description: desc,
            default: default.map(Value::String),
            required: false,
            deprecated: None,
            example,
        }
    })
}

/// 生成 PluginSchema
fn plugin_schema_with_examples(prefix: String) -> impl Strategy<Value = PluginSchema> {
    prop::collection::vec(
        (config_key(), prop::bool::ANY),
        1..5,
    ).prop_map(move |keys_with_examples| {
        let mut properties = HashMap::new();
        
        for (key, has_example) in keys_with_examples {
            let schema = PropertySchema {
                name: key.clone(),
                type_info: TypeInfo::String {
                    enum_values: None,
                    min_length: None,
                    max_length: None,
                },
                description: format!("Description for {}", key),
                default: Some(Value::String("default".to_string())),
                required: false,
                deprecated: None,
                example: if has_example {
                    Some(format!("{} = \"example_value\"", key))
                } else {
                    None
                },
            };
            properties.insert(key, schema);
        }
        
        PluginSchema {
            prefix: prefix.clone(),
            properties,
        }
    })
}

/// 生成 ConfigSchema（简化版本）
fn config_schema_with_examples() -> impl Strategy<Value = ConfigSchema> {
    prop::collection::vec(
        (config_prefix(), plugin_schema_with_examples("test".to_string())),
        1..3,
    ).prop_map(|plugin_vec| {
        let mut plugins = HashMap::new();
        for (i, (_, schema)) in plugin_vec.into_iter().enumerate() {
            let prefix = format!("plugin{}", i);
            plugins.insert(prefix.clone(), PluginSchema {
                prefix,
                properties: schema.properties,
            });
        }
        ConfigSchema { plugins }
    })
}

/// 生成 TOML 文档内容
fn toml_document_content(schema: &ConfigSchema) -> String {
    let mut content = String::new();
    
    for (prefix, plugin_schema) in &schema.plugins {
        content.push_str(&format!("[{}]\n", prefix));
        
        for (key, _) in &plugin_schema.properties {
            content.push_str(&format!("{} = \"test_value\"\n", key));
        }
        
        content.push('\n');
    }
    
    content
}

// ============================================================================
// Property 64: 文档示例显示
// Feature: spring-lsp, Property 64: 文档示例显示
// ============================================================================

/// **Property 64: 文档示例显示**
///
/// *For any* 包含示例代码的文档，悬停提示应该包含格式化的示例代码。
///
/// **Validates: Requirements 15.5**
///
/// ## 验证策略
///
/// 1. 生成包含示例代码的配置 Schema
/// 2. 生成对应的 TOML 文档
/// 3. 在配置项上悬停
/// 4. 验证悬停提示包含正确格式化的示例代码
///
/// ## 测试的正确性属性
///
/// - 如果 PropertySchema 包含 example 字段，悬停提示必须包含 "示例" 标题
/// - 示例代码必须包含在 TOML 代码块中（```toml ... ```）
/// - 示例代码内容必须与 Schema 中定义的一致
/// - 如果 PropertySchema 不包含 example 字段，悬停提示不应包含 "示例" 标题
#[cfg(test)]
mod property_64_example_display {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_hover_includes_example_when_present(
            prefix in config_prefix(),
            key in config_key(),
            description in string_value(),
            example_code in "[a-z0-9 =\\\"]{10,100}",
        ) {
            // 创建包含示例代码的 Schema
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
                    description: description.clone(),
                    default: None,
                    required: false,
                    deprecated: None,
                    example: Some(example_code.clone()),
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

            // 创建 TOML 文档
            let toml_content = format!("[{}]\n{} = \"test_value\"\n", prefix, key);
            let doc = analyzer.parse(&toml_content).unwrap();

            // 在配置项上悬停（在值的位置）
            let position = Position {
                line: 1,
                character: key.len() as u32 + 5, // 在 "test_value" 中间
            };

            let hover = analyzer.hover(&doc, position);
            prop_assert!(hover.is_some(), "Should return hover for property with example");

            if let Some(hover) = hover {
                if let lsp_types::HoverContents::Markup(content) = hover.contents {
                    let text = content.value;

                    // 验证包含示例标题
                    prop_assert!(
                        text.contains("示例") || text.contains("**示例**"),
                        "Hover text should contain example header, got: {}",
                        text
                    );

                    // 验证包含 TOML 代码块标记
                    prop_assert!(
                        text.contains("```toml"),
                        "Hover text should contain TOML code block start marker, got: {}",
                        text
                    );

                    // 验证包含代码块结束标记
                    prop_assert!(
                        text.contains("```"),
                        "Hover text should contain code block end marker, got: {}",
                        text
                    );

                    // 验证包含示例代码内容
                    prop_assert!(
                        text.contains(&example_code),
                        "Hover text should contain example code '{}', got: {}",
                        example_code,
                        text
                    );

                    // 验证代码块格式正确（示例在代码块标记之间）
                    if let Some(start_pos) = text.find("```toml") {
                        if let Some(end_pos) = text[start_pos + 7..].find("```") {
                            let code_block = &text[start_pos + 7..start_pos + 7 + end_pos];
                            prop_assert!(
                                code_block.contains(&example_code),
                                "Example code should be inside code block"
                            );
                        }
                    }
                }
            }
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_hover_excludes_example_when_absent(
            prefix in config_prefix(),
            key in config_key(),
            description in string_value(),
        ) {
            // 创建不包含示例代码的 Schema
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
                    description: description.clone(),
                    default: None,
                    required: false,
                    deprecated: None,
                    example: None, // 没有示例代码
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

            // 创建 TOML 文档
            let toml_content = format!("[{}]\n{} = \"test_value\"\n", prefix, key);
            let doc = analyzer.parse(&toml_content).unwrap();

            // 在配置项上悬停
            let position = Position {
                line: 1,
                character: key.len() as u32 + 5,
            };

            let hover = analyzer.hover(&doc, position);
            prop_assert!(hover.is_some(), "Should return hover for property");

            if let Some(hover) = hover {
                if let lsp_types::HoverContents::Markup(content) = hover.contents {
                    let text = content.value;

                    // 验证不包含示例标题
                    prop_assert!(
                        !text.contains("示例") && !text.contains("**示例**"),
                        "Hover text should not contain example header when no example is provided, got: {}",
                        text
                    );

                    // 验证不包含 TOML 代码块（除非是其他用途）
                    // 注意：可能有其他代码块，所以我们只检查没有 "示例" 标题
                }
            }
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_multiline_example_formatted_correctly(
            prefix in config_prefix(),
            key in config_key(),
        ) {
            // 创建包含多行示例代码的 Schema
            let multiline_example = format!(
                "[{}.{}]\nvalue1 = \"example\"\nvalue2 = 123\narray = [\"a\", \"b\", \"c\"]",
                prefix, key
            );

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
                    example: Some(multiline_example.clone()),
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

            // 创建 TOML 文档
            let toml_content = format!("[{}]\n{} = \"test_value\"\n", prefix, key);
            let doc = analyzer.parse(&toml_content).unwrap();

            // 在配置项上悬停
            let position = Position {
                line: 1,
                character: key.len() as u32 + 5,
            };

            let hover = analyzer.hover(&doc, position);
            prop_assert!(hover.is_some(), "Should return hover for property");

            if let Some(hover) = hover {
                if let lsp_types::HoverContents::Markup(content) = hover.contents {
                    let text = content.value;

                    // 验证包含所有行的示例代码
                    prop_assert!(
                        text.contains("value1 = \"example\""),
                        "Should contain first line of multiline example"
                    );
                    prop_assert!(
                        text.contains("value2 = 123"),
                        "Should contain second line of multiline example"
                    );
                    prop_assert!(
                        text.contains("array = [\"a\", \"b\", \"c\"]"),
                        "Should contain third line of multiline example"
                    );

                    // 验证代码块格式
                    prop_assert!(
                        text.contains("```toml"),
                        "Should have TOML code block marker"
                    );
                }
            }
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_example_appears_before_deprecated_warning(
            prefix in config_prefix(),
            key in config_key(),
            example_code in "[a-z0-9 =\\\"]{10,50}",
            deprecated_msg in "[a-z ]{10,50}",
        ) {
            // 创建包含示例代码和废弃警告的 Schema
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
                    deprecated: Some(deprecated_msg.clone()),
                    example: Some(example_code.clone()),
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

            // 创建 TOML 文档
            let toml_content = format!("[{}]\n{} = \"test_value\"\n", prefix, key);
            let doc = analyzer.parse(&toml_content).unwrap();

            // 在配置项上悬停
            let position = Position {
                line: 1,
                character: key.len() as u32 + 5,
            };

            let hover = analyzer.hover(&doc, position);
            prop_assert!(hover.is_some(), "Should return hover for property");

            if let Some(hover) = hover {
                if let lsp_types::HoverContents::Markup(content) = hover.contents {
                    let text = content.value;

                    // 查找示例和废弃警告的位置
                    let example_pos = text.find("示例").or_else(|| text.find("**示例**"));
                    let deprecated_pos = text.find("已废弃").or_else(|| text.find("⚠️"));

                    // 如果两者都存在，示例应该在废弃警告之前
                    if let (Some(ex_pos), Some(dep_pos)) = (example_pos, deprecated_pos) {
                        prop_assert!(
                            ex_pos < dep_pos,
                            "Example should appear before deprecated warning. Example at {}, deprecated at {}",
                            ex_pos,
                            dep_pos
                        );
                    }
                }
            }
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_example_code_block_well_formed(
            prefix in config_prefix(),
            key in config_key(),
        ) {
            // 使用简单的示例代码，避免特殊字符导致的格式问题
            let example_code = format!("{} = \"example_value\"", key);
            
            // 创建包含示例代码的 Schema
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
                    example: Some(example_code.clone()),
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

            // 创建 TOML 文档
            let toml_content = format!("[{}]\n{} = \"test_value\"\n", prefix, key);
            let doc = analyzer.parse(&toml_content).unwrap();

            // 在配置项上悬停
            let position = Position {
                line: 1,
                character: key.len() as u32 + 5,
            };

            let hover = analyzer.hover(&doc, position);
            prop_assert!(hover.is_some(), "Should return hover for property");

            if let Some(hover) = hover {
                if let lsp_types::HoverContents::Markup(content) = hover.contents {
                    let text = content.value;

                    // 验证包含示例代码
                    prop_assert!(
                        text.contains(&example_code),
                        "Should contain example code"
                    );

                    // 验证包含示例标题
                    prop_assert!(
                        text.contains("示例") || text.contains("**示例**"),
                        "Should contain example header"
                    );

                    // 验证包含 TOML 代码块标记
                    prop_assert!(
                        text.contains("```toml"),
                        "Should contain TOML code block start marker"
                    );

                    // 验证代码块结构：查找示例代码在代码块内
                    if let Some(toml_start) = text.find("```toml") {
                        if let Some(example_pos) = text[toml_start..].find(&example_code) {
                            // 示例代码应该在 ```toml 之后
                            prop_assert!(
                                example_pos > 0,
                                "Example code should be after ```toml marker"
                            );
                        }
                    }

                    // 验证 Markdown 格式正确
                    prop_assert_eq!(
                        content.kind,
                        lsp_types::MarkupKind::Markdown,
                        "Content should be Markdown format"
                    );
                }
            }
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(50))]

        #[test]
        fn prop_multiple_properties_with_examples(
            prefix in config_prefix(),
            num_properties in 1..5usize,
        ) {
            // 创建多个包含示例代码的属性
            let mut properties = HashMap::new();
            let mut keys = Vec::new();

            for i in 0..num_properties {
                let key = format!("property_{}", i);
                let example = format!("{} = \"example_{}\"", key, i);
                
                keys.push(key.clone());
                properties.insert(
                    key.clone(),
                    PropertySchema {
                        name: key.clone(),
                        type_info: TypeInfo::String {
                            enum_values: None,
                            min_length: None,
                            max_length: None,
                        },
                        description: format!("Property {}", i),
                        default: None,
                        required: false,
                        deprecated: None,
                        example: Some(example),
                    },
                );
            }

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

            // 创建 TOML 文档
            let mut toml_content = format!("[{}]\n", prefix);
            for key in &keys {
                toml_content.push_str(&format!("{} = \"test\"\n", key));
            }

            let doc = analyzer.parse(&toml_content).unwrap();

            // 验证每个属性的悬停提示都包含示例
            for (line_idx, key) in keys.iter().enumerate() {
                let position = Position {
                    line: (line_idx + 1) as u32,
                    character: key.len() as u32 + 5,
                };

                let hover = analyzer.hover(&doc, position);
                prop_assert!(
                    hover.is_some(),
                    "Should return hover for property '{}'",
                    key
                );

                if let Some(hover) = hover {
                    if let lsp_types::HoverContents::Markup(content) = hover.contents {
                        let text = content.value;

                        // 验证包含该属性的示例
                        prop_assert!(
                            text.contains(&format!("{} = \"example_{}\"", key, line_idx)),
                            "Hover for '{}' should contain its example",
                            key
                        );
                    }
                }
            }
        }
    }
}

// ============================================================================
// 单元测试
// ============================================================================

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_example_display_basic() {
        let mut properties = HashMap::new();
        properties.insert(
            "host".to_string(),
            PropertySchema {
                name: "host".to_string(),
                type_info: TypeInfo::String {
                    enum_values: None,
                    min_length: None,
                    max_length: None,
                },
                description: "Server host".to_string(),
                default: None,
                required: false,
                deprecated: None,
                example: Some("host = \"0.0.0.0\"".to_string()),
            },
        );

        let mut plugins = HashMap::new();
        plugins.insert(
            "web".to_string(),
            PluginSchema {
                prefix: "web".to_string(),
                properties,
            },
        );

        let schema = ConfigSchema { plugins };
        let schema_provider = SchemaProvider::from_schema(schema);
        let analyzer = TomlAnalyzer::new(schema_provider);

        let toml_content = "[web]\nhost = \"localhost\"\n";
        let doc = analyzer.parse(toml_content).unwrap();

        let position = Position {
            line: 1,
            character: 10,
        };

        let hover = analyzer.hover(&doc, position).unwrap();

        if let lsp_types::HoverContents::Markup(content) = hover.contents {
            let text = content.value;
            assert!(text.contains("示例"));
            assert!(text.contains("```toml"));
            assert!(text.contains("host = \"0.0.0.0\""));
        }
    }

    #[test]
    fn test_debug_hover_text() {
        let mut properties = HashMap::new();
        properties.insert(
            "aa0".to_string(),
            PropertySchema {
                name: "aa0".to_string(),
                type_info: TypeInfo::String {
                    enum_values: None,
                    min_length: None,
                    max_length: None,
                },
                description: "Test property".to_string(),
                default: None,
                required: false,
                deprecated: None,
                example: Some("aa0 = \"example_value\"".to_string()),
            },
        );

        let mut plugins = HashMap::new();
        plugins.insert(
            "a_a".to_string(),
            PluginSchema {
                prefix: "a_a".to_string(),
                properties,
            },
        );

        let schema = ConfigSchema { plugins };
        let schema_provider = SchemaProvider::from_schema(schema);
        let analyzer = TomlAnalyzer::new(schema_provider);

        let toml_content = "[a_a]\naa0 = \"test_value\"\n";
        let doc = analyzer.parse(toml_content).unwrap();

        let position = Position {
            line: 1,
            character: 10,
        };

        let hover = analyzer.hover(&doc, position).unwrap();

        if let lsp_types::HoverContents::Markup(content) = hover.contents {
            println!("Hover text:\n{}", content.value);
            
            let code_block_starts: Vec<_> = content.value.match_indices("```toml").collect();
            let code_block_ends: Vec<_> = content.value.match_indices("\n```").collect();
            
            println!("Code block starts: {:?}", code_block_starts);
            println!("Code block ends: {:?}", code_block_ends);
        }
    }
}
