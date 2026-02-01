//! Schema 数据模型测试

use spring_lsp::schema::*;
use std::collections::HashMap;

#[test]
fn test_config_schema_serialization() {
    // 创建一个简单的 Schema
    let mut plugins = HashMap::new();

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
            description: "服务器主机地址".to_string(),
            default: Some(Value::String("localhost".to_string())),
            required: false,
            deprecated: None,
            example: None,
        },
    );

    properties.insert(
        "port".to_string(),
        PropertySchema {
            name: "port".to_string(),
            type_info: TypeInfo::Integer {
                min: Some(1),
                max: Some(65535),
            },
            description: "服务器端口".to_string(),
            default: Some(Value::Integer(8080)),
            required: true,
            deprecated: None,
            example: None,
        },
    );

    plugins.insert(
        "web".to_string(),
        PluginSchema {
            prefix: "web".to_string(),
            properties,
        },
    );

    let schema = ConfigSchema { plugins };

    // 序列化为 JSON
    let json = serde_json::to_string_pretty(&schema).expect("序列化失败");
    println!("Schema JSON:\n{}", json);

    // 反序列化
    let deserialized: ConfigSchema = serde_json::from_str(&json).expect("反序列化失败");

    // 验证
    assert_eq!(deserialized.plugins.len(), 1);
    assert!(deserialized.plugins.contains_key("web"));

    let web_plugin = &deserialized.plugins["web"];
    assert_eq!(web_plugin.prefix, "web");
    assert_eq!(web_plugin.properties.len(), 2);

    let host_prop = &web_plugin.properties["host"];
    assert_eq!(host_prop.name, "host");
    assert_eq!(host_prop.description, "服务器主机地址");
    assert_eq!(
        host_prop.default,
        Some(Value::String("localhost".to_string()))
    );
    assert!(!host_prop.required);

    let port_prop = &web_plugin.properties["port"];
    assert_eq!(port_prop.name, "port");
    assert_eq!(port_prop.default, Some(Value::Integer(8080)));
    assert!(port_prop.required);
}

#[test]
fn test_plugin_schema_clone() {
    // 创建 PluginSchema
    let mut properties = HashMap::new();
    properties.insert(
        "enabled".to_string(),
        PropertySchema {
            name: "enabled".to_string(),
            type_info: TypeInfo::Boolean,
            description: "是否启用".to_string(),
            default: Some(Value::Boolean(true)),
            required: false,
            deprecated: None,
            example: None,
        },
    );

    let plugin = PluginSchema {
        prefix: "test".to_string(),
        properties,
    };

    // 克隆
    let cloned = plugin.clone();

    // 验证克隆的数据
    assert_eq!(cloned.prefix, "test");
    assert_eq!(cloned.properties.len(), 1);
    assert!(cloned.properties.contains_key("enabled"));
}

#[test]
fn test_property_schema_clone() {
    // 创建 PropertySchema
    let prop = PropertySchema {
        name: "timeout".to_string(),
        type_info: TypeInfo::Integer {
            min: Some(0),
            max: Some(3600),
        },
        description: "超时时间（秒）".to_string(),
        default: Some(Value::Integer(30)),
        required: false,
        deprecated: Some("使用 timeout_ms 代替".to_string()),
        example: None,
    };

    // 克隆
    let cloned = prop.clone();

    // 验证克隆的数据
    assert_eq!(cloned.name, "timeout");
    assert_eq!(cloned.description, "超时时间（秒）");
    assert_eq!(cloned.default, Some(Value::Integer(30)));
    assert!(!cloned.required);
    assert_eq!(cloned.deprecated, Some("使用 timeout_ms 代替".to_string()));
}

#[test]
fn test_type_info_variants() {
    // 测试字符串类型
    let string_type = TypeInfo::String {
        enum_values: Some(vec!["dev".to_string(), "prod".to_string()]),
        min_length: Some(1),
        max_length: Some(10),
    };

    let json = serde_json::to_string(&string_type).expect("序列化失败");
    let deserialized: TypeInfo = serde_json::from_str(&json).expect("反序列化失败");

    if let TypeInfo::String {
        enum_values,
        min_length,
        max_length,
    } = deserialized
    {
        assert_eq!(
            enum_values,
            Some(vec!["dev".to_string(), "prod".to_string()])
        );
        assert_eq!(min_length, Some(1));
        assert_eq!(max_length, Some(10));
    } else {
        panic!("类型不匹配");
    }

    // 测试整数类型
    let int_type = TypeInfo::Integer {
        min: Some(0),
        max: Some(100),
    };

    let json = serde_json::to_string(&int_type).expect("序列化失败");
    let deserialized: TypeInfo = serde_json::from_str(&json).expect("反序列化失败");

    if let TypeInfo::Integer { min, max } = deserialized {
        assert_eq!(min, Some(0));
        assert_eq!(max, Some(100));
    } else {
        panic!("类型不匹配");
    }

    // 测试浮点数类型
    let float_type = TypeInfo::Float {
        min: Some(0.0),
        max: Some(1.0),
    };

    let json = serde_json::to_string(&float_type).expect("序列化失败");
    let deserialized: TypeInfo = serde_json::from_str(&json).expect("反序列化失败");

    if let TypeInfo::Float { min, max } = deserialized {
        assert_eq!(min, Some(0.0));
        assert_eq!(max, Some(1.0));
    } else {
        panic!("类型不匹配");
    }

    // 测试布尔类型
    let bool_type = TypeInfo::Boolean;
    let json = serde_json::to_string(&bool_type).expect("序列化失败");
    let deserialized: TypeInfo = serde_json::from_str(&json).expect("反序列化失败");

    if let TypeInfo::Boolean = deserialized {
        // 成功
    } else {
        panic!("类型不匹配");
    }

    // 测试数组类型
    let array_type = TypeInfo::Array {
        item_type: Box::new(TypeInfo::String {
            enum_values: None,
            min_length: None,
            max_length: None,
        }),
    };

    let json = serde_json::to_string(&array_type).expect("序列化失败");
    let deserialized: TypeInfo = serde_json::from_str(&json).expect("反序列化失败");

    if let TypeInfo::Array { item_type } = deserialized {
        if let TypeInfo::String { .. } = *item_type {
            // 成功
        } else {
            panic!("数组元素类型不匹配");
        }
    } else {
        panic!("类型不匹配");
    }
}

#[test]
fn test_value_variants() {
    // 测试字符串值
    let string_val = Value::String("test".to_string());
    assert_eq!(string_val, Value::String("test".to_string()));

    // 测试整数值
    let int_val = Value::Integer(42);
    assert_eq!(int_val, Value::Integer(42));

    // 测试浮点数值
    let float_val = Value::Float(3.14);
    assert_eq!(float_val, Value::Float(3.14));

    // 测试布尔值
    let bool_val = Value::Boolean(true);
    assert_eq!(bool_val, Value::Boolean(true));

    // 测试数组值
    let array_val = Value::Array(vec![
        Value::String("a".to_string()),
        Value::String("b".to_string()),
    ]);
    assert_eq!(
        array_val,
        Value::Array(vec![
            Value::String("a".to_string()),
            Value::String("b".to_string()),
        ])
    );

    // 测试对象值
    let mut table = HashMap::new();
    table.insert("key".to_string(), Value::String("value".to_string()));
    let table_val = Value::Table(table.clone());
    assert_eq!(table_val, Value::Table(table));
}

#[test]
fn test_nested_object_type() {
    // 创建嵌套的对象类型
    let mut nested_props = HashMap::new();
    nested_props.insert(
        "nested_field".to_string(),
        PropertySchema {
            name: "nested_field".to_string(),
            type_info: TypeInfo::String {
                enum_values: None,
                min_length: None,
                max_length: None,
            },
            description: "嵌套字段".to_string(),
            default: None,
            required: false,
            deprecated: None,
            example: None,
        },
    );

    let object_type = TypeInfo::Object {
        properties: nested_props,
    };

    // 序列化和反序列化
    let json = serde_json::to_string(&object_type).expect("序列化失败");
    let deserialized: TypeInfo = serde_json::from_str(&json).expect("反序列化失败");

    if let TypeInfo::Object { properties } = deserialized {
        assert_eq!(properties.len(), 1);
        assert!(properties.contains_key("nested_field"));
    } else {
        panic!("类型不匹配");
    }
}

#[test]
fn test_deprecated_property() {
    // 创建废弃的属性
    let prop = PropertySchema {
        name: "old_config".to_string(),
        type_info: TypeInfo::String {
            enum_values: None,
            min_length: None,
            max_length: None,
        },
        description: "旧配置项".to_string(),
        default: None,
        required: false,
        deprecated: Some("请使用 new_config 代替".to_string()),
        example: None,
    };

    // 序列化
    let json = serde_json::to_string(&prop).expect("序列化失败");

    // 验证 JSON 包含 deprecated 字段
    assert!(json.contains("deprecated"));
    assert!(json.contains("请使用 new_config 代替"));

    // 反序列化
    let deserialized: PropertySchema = serde_json::from_str(&json).expect("反序列化失败");
    assert_eq!(
        deserialized.deprecated,
        Some("请使用 new_config 代替".to_string())
    );
}

#[test]
fn test_empty_schema() {
    // 创建空 Schema
    let schema = ConfigSchema {
        plugins: HashMap::new(),
    };

    // 序列化和反序列化
    let json = serde_json::to_string(&schema).expect("序列化失败");
    let deserialized: ConfigSchema = serde_json::from_str(&json).expect("反序列化失败");

    assert_eq!(deserialized.plugins.len(), 0);
}
