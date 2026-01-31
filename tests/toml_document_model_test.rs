//! TOML 文档模型单元测试

use spring_lsp::toml_analyzer::{
    ConfigProperty, ConfigSection, ConfigValue, EnvVarReference, TomlDocument,
};
use lsp_types::{Position, Range};
use std::collections::HashMap;

#[test]
fn test_env_var_reference_creation() {
    let env_var = EnvVarReference {
        name: "HOST".to_string(),
        default: Some("localhost".to_string()),
        range: Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: 0,
                character: 20,
            },
        },
    };

    assert_eq!(env_var.name, "HOST");
    assert_eq!(env_var.default, Some("localhost".to_string()));
}

#[test]
fn test_env_var_reference_without_default() {
    let env_var = EnvVarReference {
        name: "PORT".to_string(),
        default: None,
        range: Range {
            start: Position {
                line: 1,
                character: 0,
            },
            end: Position {
                line: 1,
                character: 10,
            },
        },
    };

    assert_eq!(env_var.name, "PORT");
    assert_eq!(env_var.default, None);
}

#[test]
fn test_config_value_types() {
    // 测试字符串值
    let string_val = ConfigValue::String("localhost".to_string());
    assert!(matches!(string_val, ConfigValue::String(_)));

    // 测试整数值
    let int_val = ConfigValue::Integer(8080);
    assert!(matches!(int_val, ConfigValue::Integer(8080)));

    // 测试浮点数值
    let float_val = ConfigValue::Float(3.14);
    assert!(matches!(float_val, ConfigValue::Float(_)));

    // 测试布尔值
    let bool_val = ConfigValue::Boolean(true);
    assert!(matches!(bool_val, ConfigValue::Boolean(true)));

    // 测试数组值
    let array_val = ConfigValue::Array(vec![
        ConfigValue::Integer(1),
        ConfigValue::Integer(2),
        ConfigValue::Integer(3),
    ]);
    assert!(matches!(array_val, ConfigValue::Array(_)));

    // 测试表值
    let mut table = HashMap::new();
    table.insert("key".to_string(), ConfigValue::String("value".to_string()));
    let table_val = ConfigValue::Table(table);
    assert!(matches!(table_val, ConfigValue::Table(_)));
}

#[test]
fn test_config_property_creation() {
    let property = ConfigProperty {
        key: "host".to_string(),
        value: ConfigValue::String("localhost".to_string()),
        range: Range {
            start: Position {
                line: 2,
                character: 0,
            },
            end: Position {
                line: 2,
                character: 20,
            },
        },
    };

    assert_eq!(property.key, "host");
    assert!(matches!(property.value, ConfigValue::String(_)));
}

#[test]
fn test_config_section_creation() {
    let mut properties = HashMap::new();
    properties.insert(
        "host".to_string(),
        ConfigProperty {
            key: "host".to_string(),
            value: ConfigValue::String("localhost".to_string()),
            range: Range {
                start: Position {
                    line: 1,
                    character: 0,
                },
                end: Position {
                    line: 1,
                    character: 20,
                },
            },
        },
    );
    properties.insert(
        "port".to_string(),
        ConfigProperty {
            key: "port".to_string(),
            value: ConfigValue::Integer(8080),
            range: Range {
                start: Position {
                    line: 2,
                    character: 0,
                },
                end: Position {
                    line: 2,
                    character: 12,
                },
            },
        },
    );

    let section = ConfigSection {
        prefix: "web".to_string(),
        properties,
        range: Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: 3,
                character: 0,
            },
        },
    };

    assert_eq!(section.prefix, "web");
    assert_eq!(section.properties.len(), 2);
    assert!(section.properties.contains_key("host"));
    assert!(section.properties.contains_key("port"));
}

#[test]
fn test_toml_document_creation() {
    // 创建一个简单的 TOML 文档用于测试
    let toml_content = "[web]\nhost = \"localhost\"\nport = 8080";
    let root = taplo::parser::parse(toml_content).into_dom();

    let mut env_vars = Vec::new();
    env_vars.push(EnvVarReference {
        name: "HOST".to_string(),
        default: Some("localhost".to_string()),
        range: Range {
            start: Position {
                line: 1,
                character: 7,
            },
            end: Position {
                line: 1,
                character: 27,
            },
        },
    });

    let mut config_sections = HashMap::new();
    let mut properties = HashMap::new();
    properties.insert(
        "host".to_string(),
        ConfigProperty {
            key: "host".to_string(),
            value: ConfigValue::String("localhost".to_string()),
            range: Range {
                start: Position {
                    line: 1,
                    character: 0,
                },
                end: Position {
                    line: 1,
                    character: 20,
                },
            },
        },
    );

    config_sections.insert(
        "web".to_string(),
        ConfigSection {
            prefix: "web".to_string(),
            properties,
            range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: 2,
                    character: 12,
                },
            },
        },
    );

    let document = TomlDocument {
        root,
        env_vars,
        config_sections,
        content: String::new(),
    };

    assert_eq!(document.env_vars.len(), 1);
    assert_eq!(document.config_sections.len(), 1);
    assert!(document.config_sections.contains_key("web"));
}

#[test]
fn test_empty_toml_document() {
    let toml_content = "";
    let root = taplo::parser::parse(toml_content).into_dom();

    let document = TomlDocument {
        root,
        env_vars: Vec::new(),
        config_sections: HashMap::new(),
        content: String::new(),
    };

    assert_eq!(document.env_vars.len(), 0);
    assert_eq!(document.config_sections.len(), 0);
}

#[test]
fn test_config_value_equality() {
    // 测试字符串值相等性
    let val1 = ConfigValue::String("test".to_string());
    let val2 = ConfigValue::String("test".to_string());
    assert_eq!(val1, val2);

    // 测试整数值相等性
    let val3 = ConfigValue::Integer(42);
    let val4 = ConfigValue::Integer(42);
    assert_eq!(val3, val4);

    // 测试布尔值相等性
    let val5 = ConfigValue::Boolean(true);
    let val6 = ConfigValue::Boolean(true);
    assert_eq!(val5, val6);

    // 测试不同类型不相等
    let val7 = ConfigValue::String("42".to_string());
    let val8 = ConfigValue::Integer(42);
    assert_ne!(val7, val8);
}

#[test]
fn test_env_var_reference_equality() {
    let env1 = EnvVarReference {
        name: "HOST".to_string(),
        default: Some("localhost".to_string()),
        range: Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: 0,
                character: 20,
            },
        },
    };

    let env2 = EnvVarReference {
        name: "HOST".to_string(),
        default: Some("localhost".to_string()),
        range: Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: 0,
                character: 20,
            },
        },
    };

    assert_eq!(env1, env2);
}

#[test]
fn test_nested_config_values() {
    // 测试嵌套的配置值
    let mut inner_table = HashMap::new();
    inner_table.insert(
        "timeout".to_string(),
        ConfigValue::Integer(30),
    );

    let mut outer_table = HashMap::new();
    outer_table.insert(
        "connection".to_string(),
        ConfigValue::Table(inner_table),
    );

    let nested_val = ConfigValue::Table(outer_table);

    if let ConfigValue::Table(outer) = nested_val {
        if let Some(ConfigValue::Table(inner)) = outer.get("connection") {
            if let Some(ConfigValue::Integer(timeout)) = inner.get("timeout") {
                assert_eq!(*timeout, 30);
            } else {
                panic!("Expected timeout to be an integer");
            }
        } else {
            panic!("Expected connection to be a table");
        }
    } else {
        panic!("Expected nested_val to be a table");
    }
}
