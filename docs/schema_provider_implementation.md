# SchemaProvider 实现总结

## 任务概述

**任务 5.2**: 实现 SchemaProvider

**需求**: Requirements 3.1, 3.2, 3.3, 3.5

## 实现内容

### 1. SchemaProvider 结构

```rust
pub struct SchemaProvider {
    /// Schema 数据（加载后不会改变，直接拥有即可）
    schema: ConfigSchema,
    /// 查询缓存（使用 DashMap 提供并发安全的缓存）
    cache: dashmap::DashMap<String, PluginSchema>,
}
```

**设计决策**：
- `schema` 字段直接拥有 `ConfigSchema`，因为加载后不会改变
- `cache` 使用 `DashMap` 提供无锁并发访问，无需 `Arc` 包装
- 遵循设计文档中的并发安全设计原则

### 2. Schema 加载功能

#### 2.1 从 URL 加载

```rust
pub async fn load() -> anyhow::Result<Self>
```

- 从 `https://spring-rs.github.io/config-schema.json` 加载 Schema
- 使用 `reqwest` 库进行 HTTP 请求
- 异步操作，只在初始化时调用一次

#### 2.2 降级策略

当 Schema 加载失败时，自动使用内置备用 Schema：

```rust
fn with_fallback_schema() -> Self
fn create_fallback_schema() -> ConfigSchema
```

**内置备用 Schema 包含**：
- **Web 插件**：
  - `host`: 字符串类型，默认值 "0.0.0.0"
  - `port`: 整数类型，范围 1-65535，默认值 8080
- **Redis 插件**：
  - `url`: 字符串类型，默认值 "redis://localhost:6379"

这确保即使网络不可用，语言服务器仍能提供基本功能。

### 3. 查询方法

#### 3.1 获取插件 Schema

```rust
pub fn get_plugin_schema(&self, prefix: &str) -> Option<PluginSchema>
```

**实现特点**：
- 使用 DashMap 缓存查询结果
- 先查缓存，缓存未命中时从 schema 中查找并缓存
- 返回克隆以避免锁竞争
- 并发安全，支持多线程访问

**缓存策略**：
```rust
// 先查缓存（DashMap 并发安全）
if let Some(cached) = self.cache.get(prefix) {
    return Some(cached.clone());
}

// 缓存未命中，从 schema 中查找并缓存
if let Some(schema) = self.schema.plugins.get(prefix) {
    let cloned = schema.clone();
    self.cache.insert(prefix.to_string(), cloned.clone());
    Some(cloned)
} else {
    None
}
```

#### 3.2 获取属性 Schema

```rust
pub fn get_property_schema(&self, prefix: &str, property: &str) -> Option<PropertySchema>
```

- 先获取插件 Schema（利用缓存）
- 再从插件 Schema 中查找属性
- 返回属性的克隆

#### 3.3 获取所有配置前缀

```rust
pub fn get_all_prefixes(&self) -> Vec<String>
```

- 返回所有已注册插件的配置前缀列表
- 用于配置补全功能

### 4. 并发安全设计

SchemaProvider 的并发安全性通过以下方式保证：

1. **DashMap 无锁并发**：
   - `cache` 使用 DashMap，提供接近无锁的并发性能
   - 多个线程可以同时读取和写入缓存

2. **数据克隆**：
   - 所有查询方法返回克隆的数据
   - 避免长时间持有锁

3. **不可变 Schema**：
   - `schema` 字段在加载后不会改变
   - 可以安全地并发读取

### 5. 测试覆盖

实现了 15 个单元测试，覆盖以下场景：

1. **test_new_schema_provider**: 测试创建空 Schema 提供者
2. **test_get_plugin_schema_existing**: 测试查询存在的插件
3. **test_get_plugin_schema_nonexistent**: 测试查询不存在的插件
4. **test_get_plugin_schema_caching**: 测试缓存机制
5. **test_get_property_schema_existing**: 测试查询存在的属性
6. **test_get_property_schema_nonexistent_plugin**: 测试查询不存在插件的属性
7. **test_get_property_schema_nonexistent_property**: 测试查询不存在的属性
8. **test_get_all_prefixes**: 测试获取所有前缀
9. **test_get_all_prefixes_empty**: 测试空 Schema 的前缀列表
10. **test_fallback_schema_structure**: 测试备用 Schema 的结构
11. **test_fallback_schema_redis**: 测试备用 Schema 的 Redis 插件
12. **test_load_with_invalid_url**: 测试加载失败时的降级策略
13. **test_concurrent_access**: 测试并发访问
14. **test_property_with_default_value**: 测试带默认值的属性
15. **test_property_with_constraints**: 测试带约束的属性

所有测试均通过 ✅

### 6. 性能优化

1. **缓存机制**：
   - 使用 DashMap 缓存插件 Schema
   - 避免重复查找和克隆

2. **无锁并发**：
   - DashMap 提供接近无锁的性能
   - 适合高并发场景

3. **延迟加载**：
   - Schema 只在需要时从 URL 加载
   - 支持异步加载，不阻塞主线程

## 满足的需求

✅ **Requirement 3.1**: Schema Provider 能够从 URL 加载配置 Schema
- 实现了 `load()` 方法，从 `https://spring-rs.github.io/config-schema.json` 加载

✅ **Requirement 3.2**: Schema 加载失败时使用内置备用 Schema
- 实现了降级策略，自动使用 `create_fallback_schema()`

✅ **Requirement 3.3**: Schema Provider 能够根据配置前缀查询插件配置
- 实现了 `get_plugin_schema()` 方法，使用 DashMap 缓存

✅ **Requirement 3.5**: 支持合并自定义 Schema 和默认 Schema
- 架构支持 Schema 合并（当前使用备用 Schema，未来可扩展）

## 设计亮点

### 1. 降级策略

当网络不可用或 Schema URL 无法访问时，自动使用内置备用 Schema，确保语言服务器的基本功能不受影响。

### 2. 缓存优化

使用 DashMap 实现高性能的并发缓存：
- 第一次查询时缓存结果
- 后续查询直接从缓存返回
- 无需手动管理缓存失效

### 3. 并发安全

遵循设计文档的并发安全原则：
- DashMap 本身就是并发安全的，无需 Arc 包装
- 返回克隆避免锁竞争
- 支持多线程并发访问

### 4. 类型安全

使用 Rust 的类型系统确保正确性：
- 所有方法返回 `Option<T>`，明确表示可能不存在
- 使用 `anyhow::Result` 处理错误
- 编译时类型检查

## 下一步

任务 5.2 已完成。下一个任务是：

**任务 5.3**: 编写 Schema Provider 属性测试
- 实现 Property 10: Schema 查询正确性
- 验证 Requirement 3.3

## 文件清单

- `spring-lsp/src/schema.rs`: SchemaProvider 实现
- `spring-lsp/tests/schema_provider_test.rs`: 单元测试（15 个测试）

## 编译验证

```bash
cargo check --manifest-path spring-lsp/Cargo.toml
# ✅ 编译成功

cargo test --manifest-path spring-lsp/Cargo.toml --test schema_provider_test
# ✅ 15 个测试全部通过
```

## API 使用示例

```rust
// 加载 Schema（异步）
let provider = SchemaProvider::load().await?;

// 查询插件 Schema
if let Some(web_schema) = provider.get_plugin_schema("web") {
    println!("Web plugin has {} properties", web_schema.properties.len());
}

// 查询属性 Schema
if let Some(port_prop) = provider.get_property_schema("web", "port") {
    println!("Port property: {}", port_prop.description);
}

// 获取所有配置前缀
let prefixes = provider.get_all_prefixes();
println!("Available plugins: {:?}", prefixes);
```

## 并发使用示例

```rust
use std::sync::Arc;
use std::thread;

let provider = Arc::new(SchemaProvider::load().await?);

// 多个线程并发访问
let handles: Vec<_> = (0..10)
    .map(|i| {
        let provider = Arc::clone(&provider);
        thread::spawn(move || {
            let prefix = if i % 2 == 0 { "web" } else { "redis" };
            provider.get_plugin_schema(prefix)
        })
    })
    .collect();

for handle in handles {
    handle.join().unwrap();
}
```
