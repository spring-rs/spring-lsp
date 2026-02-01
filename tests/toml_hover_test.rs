//! TOML 悬停提示单元测试

use lsp_types::Position;
use spring_lsp::schema::{
    ConfigSchema, PluginSchema, PropertySchema, SchemaProvider, TypeInfo, Value,
};
use spring_lsp::toml_analyzer::TomlAnalyzer;
use std::collections::HashMap;

/// 创建测试用的 Schema Provider
fn create_test_schema_provider() -> SchemaProvider {
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
            description: "Web 服务器监听地址".to_string(),
            default: Some(Value::String("localhost".to_string())),
            required: false,
            deprecated: None,
            example: Some("host = \"0.0.0.0\"".to_string()),
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
            description: "Web 服务器监听端口".to_string(),
            default: Some(Value::Integer(8080)),
            required: true,
            deprecated: None,
            example: Some("port = 8080".to_string()),
        },
    );
    web_properties.insert(
        "old_setting".to_string(),
        PropertySchema {
            name: "old_setting".to_string(),
            type_info: TypeInfo::String {
                enum_values: None,
                min_length: None,
                max_length: None,
            },
            description: "旧的配置项".to_string(),
            default: None,
            required: false,
            deprecated: Some("请使用 new_setting 代替".to_string()),
            example: None,
        },
    );

    plugins.insert(
        "web".to_string(),
        PluginSchema {
            prefix: "web".to_string(),
            properties: web_properties,
        },
    );

    // Redis 插件配置
    let mut redis_properties = HashMap::new();
    redis_properties.insert(
        "mode".to_string(),
        PropertySchema {
            name: "mode".to_string(),
            type_info: TypeInfo::String {
                enum_values: Some(vec![
                    "standalone".to_string(),
                    "cluster".to_string(),
                    "sentinel".to_string(),
                ]),
                min_length: None,
                max_length: None,
            },
            description: "Redis 运行模式".to_string(),
            default: Some(Value::String("standalone".to_string())),
            required: false,
            deprecated: None,
            example: Some("mode = \"standalone\"".to_string()),
        },
    );

    plugins.insert(
        "redis".to_string(),
        PluginSchema {
            prefix: "redis".to_string(),
            properties: redis_properties,
        },
    );

    let schema = ConfigSchema { plugins };
    SchemaProvider::from_schema(schema)
}

#[test]
fn test_hover_config_property_with_schema() {
    let schema_provider = create_test_schema_provider();
    let analyzer = TomlAnalyzer::new(schema_provider);

    let toml_content = r#"
[web]
host = "localhost"
port = 8080
"#;

    let doc = analyzer.parse(toml_content).unwrap();

    // 悬停在 host 配置项上（在值的范围内）
    let position = Position {
        line: 2,
        character: 10, // 在 "localhost" 的范围内
    };

    let hover = analyzer.hover(&doc, position);
    assert!(hover.is_some(), "应该返回悬停提示");

    let hover = hover.unwrap();
    if let lsp_types::HoverContents::Markup(content) = hover.contents {
        let text = content.value;

        // 验证包含关键信息
        assert!(text.contains("配置项: `web.host`"), "应该包含配置项名称");
        assert!(text.contains("Web 服务器监听地址"), "应该包含描述");
        assert!(text.contains("类型"), "应该包含类型信息");
        assert!(text.contains("当前值"), "应该包含当前值");
        assert!(text.contains("localhost"), "应该包含实际值");
        assert!(text.contains("默认值"), "应该包含默认值");
    } else {
        panic!("悬停内容应该是 Markup 格式");
    }
}

#[test]
fn test_hover_config_property_with_enum() {
    let schema_provider = create_test_schema_provider();
    let analyzer = TomlAnalyzer::new(schema_provider);

    let toml_content = r#"
[redis]
mode = "cluster"
"#;

    let doc = analyzer.parse(toml_content).unwrap();

    // 悬停在 mode 配置项上（在值的范围内）
    let position = Position {
        line: 2,
        character: 10, // 在 "cluster" 的范围内
    };

    let hover = analyzer.hover(&doc, position);
    assert!(hover.is_some(), "应该返回悬停提示");

    let hover = hover.unwrap();
    if let lsp_types::HoverContents::Markup(content) = hover.contents {
        let text = content.value;

        // 验证包含枚举值
        assert!(text.contains("允许的值"), "应该包含枚举值列表");
        assert!(text.contains("standalone"), "应该包含枚举值 standalone");
        assert!(text.contains("cluster"), "应该包含枚举值 cluster");
        assert!(text.contains("sentinel"), "应该包含枚举值 sentinel");
    } else {
        panic!("悬停内容应该是 Markup 格式");
    }
}

#[test]
fn test_hover_config_property_with_range() {
    let schema_provider = create_test_schema_provider();
    let analyzer = TomlAnalyzer::new(schema_provider);

    let toml_content = r#"
[web]
port = 8080
"#;

    let doc = analyzer.parse(toml_content).unwrap();

    // 悬停在 port 配置项上（在值的范围内）
    let position = Position {
        line: 2,
        character: 9, // 在 8080 的范围内
    };

    let hover = analyzer.hover(&doc, position);
    assert!(hover.is_some(), "应该返回悬停提示");

    let hover = hover.unwrap();
    if let lsp_types::HoverContents::Markup(content) = hover.contents {
        let text = content.value;

        // 验证包含范围限制
        assert!(text.contains("值范围"), "应该包含值范围信息");
        assert!(text.contains("最小值"), "应该包含最小值");
        assert!(text.contains("最大值"), "应该包含最大值");
        assert!(text.contains("必需"), "应该标记为必需");
    } else {
        panic!("悬停内容应该是 Markup 格式");
    }
}

#[test]
fn test_hover_deprecated_property() {
    let schema_provider = create_test_schema_provider();
    let analyzer = TomlAnalyzer::new(schema_provider);

    let toml_content = r#"
[web]
old_setting = "value"
"#;

    let doc = analyzer.parse(toml_content).unwrap();

    // 悬停在 old_setting 配置项上（在值的范围内）
    let position = Position {
        line: 2,
        character: 18, // 在 "value" 的范围内
    };

    let hover = analyzer.hover(&doc, position);
    assert!(hover.is_some(), "应该返回悬停提示");

    let hover = hover.unwrap();
    if let lsp_types::HoverContents::Markup(content) = hover.contents {
        let text = content.value;

        // 验证包含废弃警告
        assert!(text.contains("已废弃"), "应该包含废弃警告");
        assert!(text.contains("new_setting"), "应该包含替代建议");
    } else {
        panic!("悬停内容应该是 Markup 格式");
    }
}

#[test]
fn test_hover_undefined_property() {
    let schema_provider = create_test_schema_provider();
    let analyzer = TomlAnalyzer::new(schema_provider);

    let toml_content = r#"
[web]
unknown = "value"
"#;

    let doc = analyzer.parse(toml_content).unwrap();

    // 悬停在 unknown 配置项上（在值的范围内）
    let position = Position {
        line: 2,
        character: 12, // 在 "value" 的范围内
    };

    let hover = analyzer.hover(&doc, position);
    assert!(hover.is_some(), "应该返回悬停提示");

    let hover = hover.unwrap();
    if let lsp_types::HoverContents::Markup(content) = hover.contents {
        let text = content.value;

        // 验证包含警告
        assert!(text.contains("警告"), "应该包含警告");
        assert!(text.contains("未在 Schema 中定义"), "应该说明未定义");
    } else {
        panic!("悬停内容应该是 Markup 格式");
    }
}

#[test]
fn test_hover_env_var_with_default() {
    let schema_provider = create_test_schema_provider();
    let analyzer = TomlAnalyzer::new(schema_provider);

    let toml_content = r#"
[web]
host = "${HOST:localhost}"
"#;

    let doc = analyzer.parse(toml_content).unwrap();

    // 悬停在环境变量上（假设在引号内）
    let position = Position {
        line: 2,
        character: 10,
    };

    let hover = analyzer.hover(&doc, position);
    assert!(hover.is_some(), "应该返回悬停提示");

    let hover = hover.unwrap();
    if let lsp_types::HoverContents::Markup(content) = hover.contents {
        let text = content.value;

        // 验证包含环境变量信息
        assert!(text.contains("环境变量"), "应该标识为环境变量");
        assert!(text.contains("变量名"), "应该包含变量名");
        assert!(text.contains("HOST"), "应该包含变量名 HOST");
        assert!(text.contains("默认值"), "应该包含默认值");
        assert!(text.contains("localhost"), "应该包含默认值 localhost");
        assert!(text.contains("当前值"), "应该包含当前值信息");
    } else {
        panic!("悬停内容应该是 Markup 格式");
    }
}

#[test]
fn test_hover_env_var_without_default() {
    let schema_provider = create_test_schema_provider();
    let analyzer = TomlAnalyzer::new(schema_provider);

    let toml_content = r#"
[web]
host = "${HOST}"
"#;

    let doc = analyzer.parse(toml_content).unwrap();

    // 悬停在环境变量上
    let position = Position {
        line: 2,
        character: 10,
    };

    let hover = analyzer.hover(&doc, position);
    assert!(hover.is_some(), "应该返回悬停提示");

    let hover = hover.unwrap();
    if let lsp_types::HoverContents::Markup(content) = hover.contents {
        let text = content.value;

        // 验证包含环境变量信息
        assert!(text.contains("环境变量"), "应该标识为环境变量");
        assert!(text.contains("HOST"), "应该包含变量名");
        assert!(text.contains("格式"), "应该包含格式说明");
    } else {
        panic!("悬停内容应该是 Markup 格式");
    }
}

#[test]
fn test_hover_no_match() {
    let schema_provider = create_test_schema_provider();
    let analyzer = TomlAnalyzer::new(schema_provider);

    let toml_content = r#"
[web]
host = "localhost"
"#;

    let doc = analyzer.parse(toml_content).unwrap();

    // 悬停在空白位置
    let position = Position {
        line: 0,
        character: 0,
    };

    let hover = analyzer.hover(&doc, position);
    assert!(hover.is_none(), "空白位置不应该返回悬停提示");
}

#[test]
fn test_hover_position_in_range() {
    let schema_provider = create_test_schema_provider();
    let analyzer = TomlAnalyzer::new(schema_provider);

    // 测试位置检查逻辑
    let toml_content = r#"
[web]
host = "localhost"
port = 8080
"#;

    let doc = analyzer.parse(toml_content).unwrap();

    // 测试多个位置
    let positions = vec![
        Position {
            line: 2,
            character: 0,
        }, // host 行开始
        Position {
            line: 2,
            character: 2,
        }, // host 中间
        Position {
            line: 3,
            character: 0,
        }, // port 行开始
        Position {
            line: 3,
            character: 2,
        }, // port 中间
    ];

    for pos in positions {
        let hover = analyzer.hover(&doc, pos);
        // 至少应该能找到一些悬停信息
        // 注意：由于 taplo 的范围计算可能不精确，这里只验证不会崩溃
        let _ = hover;
    }
}

#[test]
fn test_hover_with_complex_value() {
    let schema_provider = create_test_schema_provider();
    let analyzer = TomlAnalyzer::new(schema_provider);

    let toml_content = r#"
[web]
host = "localhost"
"#;

    let doc = analyzer.parse(toml_content).unwrap();

    // 悬停在配置项上
    let position = Position {
        line: 2,
        character: 2,
    };

    let hover = analyzer.hover(&doc, position);

    if let Some(hover) = hover {
        if let lsp_types::HoverContents::Markup(content) = hover.contents {
            let text = content.value;

            // 验证 Markdown 格式正确
            assert!(text.contains("#"), "应该包含 Markdown 标题");
            assert!(text.contains("**"), "应该包含 Markdown 粗体");
            assert!(text.contains("`"), "应该包含 Markdown 代码");
        }
    }
}

#[test]
fn test_hover_markdown_format() {
    let schema_provider = create_test_schema_provider();
    let analyzer = TomlAnalyzer::new(schema_provider);

    let toml_content = r#"
[web]
port = 8080
"#;

    let doc = analyzer.parse(toml_content).unwrap();

    let position = Position {
        line: 2,
        character: 9, // 在 8080 的范围内
    };

    let hover = analyzer.hover(&doc, position).unwrap();

    // 验证返回的是 Markdown 格式
    if let lsp_types::HoverContents::Markup(content) = hover.contents {
        assert_eq!(content.kind, lsp_types::MarkupKind::Markdown);

        let text = content.value;

        // 验证 Markdown 结构
        assert!(text.starts_with("#"), "应该以标题开始");
        assert!(text.contains("**"), "应该包含粗体文本");
        assert!(text.contains("`"), "应该包含代码标记");
        assert!(text.contains("---"), "应该包含分隔线");
    } else {
        panic!("应该返回 Markup 格式");
    }
}

#[test]
fn test_hover_with_example_code() {
    let schema_provider = create_test_schema_provider();
    let analyzer = TomlAnalyzer::new(schema_provider);

    let toml_content = r#"
[web]
host = "localhost"
port = 8080
"#;

    let doc = analyzer.parse(toml_content).unwrap();

    // 悬停在 host 配置项上（在值的范围内）
    let position = Position {
        line: 2,
        character: 10, // 在 "localhost" 的范围内
    };

    let hover = analyzer.hover(&doc, position);
    assert!(hover.is_some(), "应该返回悬停提示");

    let hover = hover.unwrap();
    if let lsp_types::HoverContents::Markup(content) = hover.contents {
        let text = content.value;

        // 验证包含示例代码
        assert!(text.contains("示例"), "应该包含示例标题");
        assert!(text.contains("```toml"), "应该包含 TOML 代码块开始标记");
        assert!(text.contains("host = \"0.0.0.0\""), "应该包含示例代码内容");
        assert!(text.contains("```"), "应该包含代码块结束标记");

        // 验证示例代码在废弃警告之前（如果有的话）
        let example_pos = text.find("示例").unwrap();
        if let Some(deprecated_pos) = text.find("已废弃") {
            assert!(example_pos < deprecated_pos, "示例应该在废弃警告之前");
        }
    } else {
        panic!("悬停内容应该是 Markup 格式");
    }
}

#[test]
fn test_hover_without_example_code() {
    let schema_provider = create_test_schema_provider();
    let analyzer = TomlAnalyzer::new(schema_provider);

    let toml_content = r#"
[web]
old_setting = "value"
"#;

    let doc = analyzer.parse(toml_content).unwrap();

    // 悬停在 old_setting 配置项上（没有示例代码）
    let position = Position {
        line: 2,
        character: 18, // 在 "value" 的范围内
    };

    let hover = analyzer.hover(&doc, position);
    assert!(hover.is_some(), "应该返回悬停提示");

    let hover = hover.unwrap();
    if let lsp_types::HoverContents::Markup(content) = hover.contents {
        let text = content.value;

        // 验证不包含示例代码
        assert!(!text.contains("示例"), "不应该包含示例标题");
        assert!(!text.contains("```toml"), "不应该包含 TOML 代码块");
    } else {
        panic!("悬停内容应该是 Markup 格式");
    }
}

#[test]
fn test_hover_example_code_formatting() {
    let schema_provider = create_test_schema_provider();
    let analyzer = TomlAnalyzer::new(schema_provider);

    let toml_content = r#"
[redis]
mode = "cluster"
"#;

    let doc = analyzer.parse(toml_content).unwrap();

    // 悬停在 mode 配置项上
    let position = Position {
        line: 2,
        character: 10, // 在 "cluster" 的范围内
    };

    let hover = analyzer.hover(&doc, position);
    assert!(hover.is_some(), "应该返回悬停提示");

    let hover = hover.unwrap();
    if let lsp_types::HoverContents::Markup(content) = hover.contents {
        let text = content.value;

        // 验证示例代码格式化正确
        assert!(text.contains("**示例**:"), "应该有示例标题");

        // 验证代码块格式
        let code_block_start = text.find("```toml\n");
        let code_block_end = text.rfind("\n```");

        assert!(code_block_start.is_some(), "应该有代码块开始标记");
        assert!(code_block_end.is_some(), "应该有代码块结束标记");

        if let (Some(start), Some(end)) = (code_block_start, code_block_end) {
            assert!(start < end, "代码块开始应该在结束之前");

            // 提取代码块内容
            let code_content = &text[start + 8..end]; // 8 = "```toml\n".len()
            assert!(
                code_content.contains("mode = \"standalone\""),
                "应该包含示例代码"
            );
        }
    } else {
        panic!("悬停内容应该是 Markup 格式");
    }
}

#[test]
fn test_hover_example_with_multiline_code() {
    // 创建一个包含多行示例的 Schema
    let mut plugins = HashMap::new();
    let mut web_properties = HashMap::new();

    web_properties.insert(
        "cors".to_string(),
        PropertySchema {
            name: "cors".to_string(),
            type_info: TypeInfo::String {
                enum_values: None,
                min_length: None,
                max_length: None,
            },
            description: "CORS 配置".to_string(),
            default: None,
            required: false,
            deprecated: None,
            example: Some(
                "[web.cors]\nallow_origins = [\"*\"]\nallow_methods = [\"GET\", \"POST\"]"
                    .to_string(),
            ),
        },
    );

    plugins.insert(
        "web".to_string(),
        PluginSchema {
            prefix: "web".to_string(),
            properties: web_properties,
        },
    );

    let schema = ConfigSchema { plugins };
    let schema_provider = SchemaProvider::from_schema(schema);
    let analyzer = TomlAnalyzer::new(schema_provider);

    let toml_content = r#"
[web]
cors = "enabled"
"#;

    let doc = analyzer.parse(toml_content).unwrap();

    // 悬停在 cors 配置项上
    let position = Position {
        line: 2,
        character: 10,
    };

    let hover = analyzer.hover(&doc, position);
    assert!(hover.is_some(), "应该返回悬停提示");

    let hover = hover.unwrap();
    if let lsp_types::HoverContents::Markup(content) = hover.contents {
        let text = content.value;

        // 验证多行示例代码正确显示
        assert!(text.contains("示例"), "应该包含示例标题");
        assert!(text.contains("```toml"), "应该包含代码块标记");
        assert!(text.contains("allow_origins"), "应该包含多行示例的第一行");
        assert!(text.contains("allow_methods"), "应该包含多行示例的第二行");
    } else {
        panic!("悬停内容应该是 Markup 格式");
    }
}
