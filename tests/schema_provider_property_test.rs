//! SchemaProvider 属性测试
//!
//! 使用 proptest 验证 SchemaProvider 的通用正确性属性

use proptest::prelude::*;
use spring_lsp::schema::{ConfigSchema, PluginSchema, PropertySchema, SchemaProvider, TypeInfo, Value};
use std::collections::HashMap;

// ============================================================================
// 测试数据生成器
// ============================================================================

/// 生成有效的配置前缀
/// 
/// 配置前缀应该是小写字母和连字符组成的字符串
fn valid_prefix() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9-]{0,20}"
}

/// 生成有效的属性名称
/// 
/// 属性名称应该是小写字母、数字和下划线组成的字符串
fn valid_property_name() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,30}"
}

/// 生成有效的描述文本
fn valid_description() -> impl Strategy<Value = String> {
    "[A-Za-z0-9 .,!?-]{1,100}"
}

/// 生成 TypeInfo
fn type_info() -> impl Strategy<Value = TypeInfo> {
    prop_oneof![
        // String 类型
        Just(TypeInfo::String {
            enum_values: None,
            min_length: None,
            max_length: None,
        }),
        // Integer 类型
        Just(TypeInfo::Integer {
            min: Some(0),
            max: Some(100),
        }),
        // Float 类型
        Just(TypeInfo::Float {
            min: Some(0.0),
            max: Some(100.0),
        }),
        // Boolean 类型
        Just(TypeInfo::Boolean),
    ]
}

/// 生成 Value
fn value() -> impl Strategy<Value = Value> {
    prop_oneof![
        any::<String>().prop_map(Value::String),
        any::<i64>().prop_map(Value::Integer),
        any::<f64>().prop_filter("finite float", |f| f.is_finite()).prop_map(Value::Float),
        any::<bool>().prop_map(Value::Boolean),
    ]
}

/// 生成 PropertySchema
fn property_schema() -> impl Strategy<Value = PropertySchema> {
    (
        valid_property_name(),
        type_info(),
        valid_description(),
        proptest::option::of(value()),
        any::<bool>(),
        proptest::option::of("[A-Za-z0-9 .,!?-]{1,50}"),
    )
        .prop_map(|(name, type_info, description, default, required, deprecated)| {
            PropertySchema {
                name: name.clone(),
                type_info,
                description,
                default,
                required,
                deprecated,
                example: None,
            }
        })
}

/// 生成 PluginSchema
fn plugin_schema() -> impl Strategy<Value = PluginSchema> {
    (
        valid_prefix(),
        prop::collection::hash_map(valid_property_name(), property_schema(), 0..10),
    )
        .prop_map(|(prefix, properties)| PluginSchema { prefix, properties })
}

/// 生成 ConfigSchema
fn config_schema() -> impl Strategy<Value = ConfigSchema> {
    prop::collection::hash_map(valid_prefix(), plugin_schema(), 0..10)
        .prop_map(|plugins| {
            // 确保每个 PluginSchema 的 prefix 字段与 HashMap 的键匹配
            let mut corrected_plugins = HashMap::new();
            for (key, mut plugin) in plugins {
                plugin.prefix = key.clone();
                corrected_plugins.insert(key, plugin);
            }
            ConfigSchema { plugins: corrected_plugins }
        })
}

// ============================================================================
// 属性测试
// ============================================================================

// Feature: spring-lsp, Property 10: Schema 查询正确性
// 
// **Validates: Requirements 3.3**
// 
// *For any* 在 Schema 中定义的配置前缀，Schema Provider 应该返回对应的插件配置定义。
// 
// 这个属性测试验证：
// 1. 对于 Schema 中存在的任何前缀，get_plugin_schema 应该返回 Some
// 2. 返回的 PluginSchema 的 prefix 字段应该与查询的前缀匹配
// 3. 返回的 PluginSchema 应该包含正确的属性
// 4. 对于 Schema 中不存在的前缀，get_plugin_schema 应该返回 None
proptest! {
    #[test]
    fn prop_schema_query_correctness(schema in config_schema()) {
        // 创建 SchemaProvider（使用生成的 schema）
        let provider = SchemaProvider::from_schema(schema.clone());
        
        // 属性 1: 对于 Schema 中存在的任何前缀，应该能够查询到
        for (prefix, expected_plugin) in &schema.plugins {
            let result = provider.get_plugin_schema(prefix);
            
            // 应该返回 Some
            prop_assert!(result.is_some(), 
                "Should find plugin schema for prefix '{}'", prefix);
            
            let plugin = result.unwrap();
            
            // 返回的 prefix 应该匹配
            prop_assert_eq!(&plugin.prefix, prefix,
                "Returned plugin prefix should match query");
            
            // 返回的属性数量应该匹配
            prop_assert_eq!(plugin.properties.len(), expected_plugin.properties.len(),
                "Returned plugin should have same number of properties");
            
            // 验证每个属性
            for (prop_name, expected_prop) in &expected_plugin.properties {
                prop_assert!(plugin.properties.contains_key(prop_name),
                    "Plugin should contain property '{}'", prop_name);
                
                let prop = plugin.properties.get(prop_name).unwrap();
                prop_assert_eq!(&prop.name, &expected_prop.name,
                    "Property name should match");
            }
        }
        
        // 属性 2: 对于 Schema 中不存在的前缀，应该返回 None
        let nonexistent_prefix = "nonexistent-plugin-xyz-123";
        if !schema.plugins.contains_key(nonexistent_prefix) {
            let result = provider.get_plugin_schema(nonexistent_prefix);
            prop_assert!(result.is_none(),
                "Should not find schema for nonexistent prefix");
        }
    }
}

// Feature: spring-lsp, Property 10: Schema 查询正确性（属性查询）
// 
// **Validates: Requirements 3.3**
// 
// 验证 get_property_schema 方法的正确性
proptest! {
    #[test]
    fn prop_property_query_correctness(schema in config_schema()) {
        let provider = SchemaProvider::from_schema(schema.clone());
        
        // 对于 Schema 中存在的每个插件和属性，应该能够查询到
        for (prefix, plugin) in &schema.plugins {
            for (prop_name, expected_prop) in &plugin.properties {
                let result = provider.get_property_schema(prefix, prop_name);
                
                // 应该返回 Some
                prop_assert!(result.is_some(),
                    "Should find property '{}' in plugin '{}'", prop_name, prefix);
                
                let prop = result.unwrap();
                
                // 属性名称应该匹配
                prop_assert_eq!(&prop.name, &expected_prop.name,
                    "Property name should match");
                
                // 描述应该匹配
                prop_assert_eq!(&prop.description, &expected_prop.description,
                    "Property description should match");
                
                // required 标志应该匹配
                prop_assert_eq!(prop.required, expected_prop.required,
                    "Property required flag should match");
            }
        }
        
        // 对于不存在的插件，应该返回 None
        let result = provider.get_property_schema("nonexistent-plugin", "any-property");
        prop_assert!(result.is_none(),
            "Should not find property in nonexistent plugin");
        
        // 对于存在的插件但不存在的属性，应该返回 None
        if let Some((prefix, _)) = schema.plugins.iter().next() {
            let result = provider.get_property_schema(prefix, "nonexistent-property-xyz");
            prop_assert!(result.is_none(),
                "Should not find nonexistent property in existing plugin");
        }
    }
}

// Feature: spring-lsp, Property 10: Schema 查询正确性（前缀列表）
// 
// **Validates: Requirements 3.3**
// 
// 验证 get_all_prefixes 方法的正确性
proptest! {
    #[test]
    fn prop_prefix_list_correctness(schema in config_schema()) {
        let provider = SchemaProvider::from_schema(schema.clone());
        
        let prefixes = provider.get_all_prefixes();
        
        // 返回的前缀数量应该与 Schema 中的插件数量匹配
        prop_assert_eq!(prefixes.len(), schema.plugins.len(),
            "Number of prefixes should match number of plugins");
        
        // 每个 Schema 中的前缀都应该在返回的列表中
        for prefix in schema.plugins.keys() {
            prop_assert!(prefixes.contains(prefix),
                "Prefix list should contain '{}'", prefix);
        }
        
        // 返回的列表中不应该有重复
        let unique_count = prefixes.iter().collect::<std::collections::HashSet<_>>().len();
        prop_assert_eq!(unique_count, prefixes.len(),
            "Prefix list should not contain duplicates");
    }
}

// Feature: spring-lsp, Property 10: Schema 查询正确性（缓存一致性）
// 
// **Validates: Requirements 3.3**
// 
// 验证缓存机制不会影响查询结果的正确性
proptest! {
    #[test]
    fn prop_cache_consistency(schema in config_schema()) {
        let provider = SchemaProvider::from_schema(schema.clone());
        
        // 对于每个插件，多次查询应该返回相同的结果
        for prefix in schema.plugins.keys() {
            let result1 = provider.get_plugin_schema(prefix);
            let result2 = provider.get_plugin_schema(prefix);
            let result3 = provider.get_plugin_schema(prefix);
            
            // 所有查询都应该成功
            prop_assert!(result1.is_some());
            prop_assert!(result2.is_some());
            prop_assert!(result3.is_some());
            
            let plugin1 = result1.unwrap();
            let plugin2 = result2.unwrap();
            let plugin3 = result3.unwrap();
            
            // 所有查询返回的 prefix 应该相同
            prop_assert_eq!(&plugin1.prefix, &plugin2.prefix);
            prop_assert_eq!(&plugin2.prefix, &plugin3.prefix);
            
            // 所有查询返回的属性数量应该相同
            prop_assert_eq!(plugin1.properties.len(), plugin2.properties.len());
            prop_assert_eq!(plugin2.properties.len(), plugin3.properties.len());
        }
    }
}

// Feature: spring-lsp, Property 10: Schema 查询正确性（空 Schema）
// 
// **Validates: Requirements 3.3**
// 
// 验证空 Schema 的边缘情况
proptest! {
    #[test]
    fn prop_empty_schema_handling(prefix in valid_prefix()) {
        // 创建空 Schema
        let empty_schema = ConfigSchema {
            plugins: HashMap::new(),
        };
        let provider = SchemaProvider::from_schema(empty_schema);
        
        // 查询任何前缀都应该返回 None
        let result = provider.get_plugin_schema(&prefix);
        prop_assert!(result.is_none(),
            "Empty schema should not return any plugin");
        
        // get_all_prefixes 应该返回空列表
        let prefixes = provider.get_all_prefixes();
        prop_assert_eq!(prefixes.len(), 0,
            "Empty schema should return empty prefix list");
    }
}
