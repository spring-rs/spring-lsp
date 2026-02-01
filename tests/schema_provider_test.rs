//! SchemaProvider 单元测试

use spring_lsp::schema::{SchemaProvider, TypeInfo, Value};

#[test]
fn test_new_schema_provider() {
    let provider = SchemaProvider::new();
    let prefixes = provider.get_all_prefixes();
    assert_eq!(prefixes.len(), 0, "New provider should have no plugins");
}

#[test]
fn test_get_plugin_schema_existing() {
    let _provider = SchemaProvider::new();
    // 使用 with_fallback_schema 创建带有内置 Schema 的提供者
    let provider = SchemaProvider::default();

    // 注意：fallback schema 包含 web 和 redis
    let web_schema = provider.get_plugin_schema("web");
    assert!(web_schema.is_some(), "Should find web plugin schema");

    let schema = web_schema.unwrap();
    assert_eq!(schema.prefix, "web");
    assert!(schema.properties.contains_key("host"));
    assert!(schema.properties.contains_key("port"));
}

#[test]
fn test_get_plugin_schema_nonexistent() {
    let provider = SchemaProvider::new();
    let schema = provider.get_plugin_schema("nonexistent");
    assert!(schema.is_none(), "Should not find nonexistent plugin");
}

#[test]
fn test_get_plugin_schema_caching() {
    let provider = SchemaProvider::default();

    // 第一次查询（缓存未命中）
    let schema1 = provider.get_plugin_schema("web");
    assert!(schema1.is_some());

    // 第二次查询（应该从缓存返回）
    let schema2 = provider.get_plugin_schema("web");
    assert!(schema2.is_some());

    // 验证返回的是相同的数据
    assert_eq!(schema1.unwrap().prefix, schema2.unwrap().prefix);
}

#[test]
fn test_get_property_schema_existing() {
    let provider = SchemaProvider::default();

    let property = provider.get_property_schema("web", "host");
    assert!(property.is_some(), "Should find host property");

    let prop = property.unwrap();
    assert_eq!(prop.name, "host");
    assert_eq!(prop.description, "Web server host address");
}

#[test]
fn test_get_property_schema_nonexistent_plugin() {
    let provider = SchemaProvider::default();

    let property = provider.get_property_schema("nonexistent", "host");
    assert!(
        property.is_none(),
        "Should not find property in nonexistent plugin"
    );
}

#[test]
fn test_get_property_schema_nonexistent_property() {
    let provider = SchemaProvider::default();

    let property = provider.get_property_schema("web", "nonexistent");
    assert!(property.is_none(), "Should not find nonexistent property");
}

#[test]
fn test_get_all_prefixes() {
    let provider = SchemaProvider::default();

    let prefixes = provider.get_all_prefixes();
    assert!(
        prefixes.len() >= 2,
        "Fallback schema should have at least 2 plugins"
    );
    assert!(prefixes.contains(&"web".to_string()));
    assert!(prefixes.contains(&"redis".to_string()));
}

#[test]
fn test_get_all_prefixes_empty() {
    let provider = SchemaProvider::new();

    let prefixes = provider.get_all_prefixes();
    assert_eq!(prefixes.len(), 0, "New provider should have no prefixes");
}

#[test]
fn test_fallback_schema_structure() {
    let provider = SchemaProvider::default();

    // 验证 web 插件
    let web_schema = provider.get_plugin_schema("web").unwrap();
    assert_eq!(web_schema.prefix, "web");
    assert!(web_schema.properties.contains_key("host"));
    assert!(web_schema.properties.contains_key("port"));

    // 验证 host 属性
    let host_prop = web_schema.properties.get("host").unwrap();
    assert_eq!(host_prop.name, "host");
    assert!(!host_prop.required);
    assert!(matches!(host_prop.type_info, TypeInfo::String { .. }));

    // 验证 port 属性
    let port_prop = web_schema.properties.get("port").unwrap();
    assert_eq!(port_prop.name, "port");
    assert!(!port_prop.required);
    if let TypeInfo::Integer { min, max } = &port_prop.type_info {
        assert_eq!(*min, Some(1));
        assert_eq!(*max, Some(65535));
    } else {
        panic!("port should be Integer type");
    }
}

#[test]
fn test_fallback_schema_redis() {
    let provider = SchemaProvider::default();

    // 验证 redis 插件
    let redis_schema = provider.get_plugin_schema("redis").unwrap();
    assert_eq!(redis_schema.prefix, "redis");
    assert!(redis_schema.properties.contains_key("url"));

    // 验证 url 属性
    let url_prop = redis_schema.properties.get("url").unwrap();
    assert_eq!(url_prop.name, "url");
    assert!(matches!(url_prop.type_info, TypeInfo::String { .. }));
}

#[tokio::test]
async fn test_load_with_invalid_url() {
    // 使用无效的 URL，应该降级到备用 Schema
    let provider = SchemaProvider::load().await;

    // 即使加载失败，也应该返回 Ok（使用备用 Schema）
    assert!(provider.is_ok());

    let provider = provider.unwrap();
    let prefixes = provider.get_all_prefixes();

    // 备用 Schema 应该包含一些插件
    assert!(prefixes.len() >= 2);
}

#[test]
fn test_concurrent_access() {
    use std::sync::Arc;
    use std::thread;

    let provider = Arc::new(SchemaProvider::default());
    let mut handles = vec![];

    // 创建多个线程并发访问
    for i in 0..10 {
        let provider_clone = Arc::clone(&provider);
        let handle = thread::spawn(move || {
            let prefix = if i % 2 == 0 { "web" } else { "redis" };
            let schema = provider_clone.get_plugin_schema(prefix);
            assert!(schema.is_some());
        });
        handles.push(handle);
    }

    // 等待所有线程完成
    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_property_with_default_value() {
    let provider = SchemaProvider::default();

    let property = provider.get_property_schema("web", "host").unwrap();
    assert!(property.default.is_some());

    if let Some(Value::String(default_host)) = property.default {
        assert_eq!(default_host, "0.0.0.0");
    } else {
        panic!("host should have string default value");
    }
}

#[test]
fn test_property_with_constraints() {
    let provider = SchemaProvider::default();

    let property = provider.get_property_schema("web", "port").unwrap();

    if let TypeInfo::Integer { min, max } = property.type_info {
        assert_eq!(min, Some(1));
        assert_eq!(max, Some(65535));
    } else {
        panic!("port should be Integer type with constraints");
    }
}
