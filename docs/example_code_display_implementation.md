# 示例代码显示功能实现文档

## 概述

本文档描述了 spring-lsp 语言服务器中示例代码显示功能的实现细节。该功能允许在悬停提示中显示配置项的示例代码，帮助开发者快速了解配置项的正确用法。

## 实现的功能

### 1. Schema 扩展

在 `PropertySchema` 结构体中添加了 `example` 字段：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertySchema {
    // ... 其他字段 ...
    
    /// 示例代码（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<String>,
}
```

该字段用于存储配置项的示例代码，支持单行或多行 TOML 代码。

### 2. 悬停提示增强

更新了 `TomlAnalyzer::create_property_hover()` 方法，在悬停提示中显示示例代码：

```rust
// 添加示例代码（如果有）
if let Some(example) = &schema.example {
    hover_text.push_str("**示例**:\n\n");
    hover_text.push_str("```toml\n");
    hover_text.push_str(example);
    hover_text.push_str("\n```\n\n");
}
```

示例代码使用 Markdown 代码块格式化，并指定 `toml` 语言标识符以启用语法高亮。

### 3. 示例代码位置

示例代码在悬停提示中的显示顺序：

1. 配置项标题
2. 描述
3. 类型信息
4. 当前值
5. 默认值
6. 必需性
7. 枚举值（如果有）
8. 值范围限制（如果有）
9. **示例代码**（新增）
10. 废弃警告（如果有）
11. 配置文件位置提示

示例代码放在废弃警告之前，确保用户首先看到正确的用法示例。

## 示例

### 单行示例

```rust
PropertySchema {
    name: "host".to_string(),
    // ... 其他字段 ...
    example: Some("host = \"0.0.0.0\"".to_string()),
}
```

悬停提示显示：

```markdown
**示例**:

```toml
host = "0.0.0.0"
```
```

### 多行示例

```rust
PropertySchema {
    name: "cors".to_string(),
    // ... 其他字段 ...
    example: Some("[web.cors]\nallow_origins = [\"*\"]\nallow_methods = [\"GET\", \"POST\"]".to_string()),
}
```

悬停提示显示：

```markdown
**示例**:

```toml
[web.cors]
allow_origins = ["*"]
allow_methods = ["GET", "POST"]
```
```

## 测试覆盖

实现了以下单元测试：

1. **test_hover_with_example_code**: 验证包含示例代码的配置项悬停提示
2. **test_hover_without_example_code**: 验证不包含示例代码的配置项悬停提示
3. **test_hover_example_code_formatting**: 验证示例代码的格式化正确性
4. **test_hover_example_with_multiline_code**: 验证多行示例代码的显示
5. 更新了所有现有测试以包含 `example` 字段

所有测试均通过，确保功能正确性。

## 使用指南

### 为配置项添加示例

在创建 `PropertySchema` 时，添加 `example` 字段：

```rust
PropertySchema {
    name: "port".to_string(),
    type_info: TypeInfo::Integer {
        min: Some(1),
        max: Some(65535),
    },
    description: "Web 服务器监听端口".to_string(),
    default: Some(Value::Integer(8080)),
    required: false,
    deprecated: None,
    example: Some("port = 8080".to_string()),  // 添加示例
}
```

### 示例代码编写建议

1. **简洁明了**: 示例应该简短且易于理解
2. **完整有效**: 示例应该是有效的 TOML 代码
3. **展示用法**: 示例应该展示配置项的典型用法
4. **包含上下文**: 对于复杂配置，可以包含相关的配置节

### 示例代码格式

- **单行配置**: `key = value`
- **多行配置**: 使用 `\n` 分隔多行
- **嵌套配置**: 包含配置节标题，如 `[section]\nkey = value`

## 技术细节

### Markdown 格式化

示例代码使用 Markdown 代码块格式：

```markdown
**示例**:

```toml
<example_code>
```
```

这确保了：
- 代码块有明确的开始和结束标记
- 指定了 `toml` 语言标识符以启用语法高亮
- 在支持的编辑器中提供良好的视觉效果

### 序列化和反序列化

`example` 字段使用 `#[serde(skip_serializing_if = "Option::is_none")]` 属性：

- 当 `example` 为 `None` 时，不会在 JSON Schema 中序列化该字段
- 减少 Schema 文件大小
- 保持向后兼容性

### 性能考虑

- 示例代码存储在 Schema 中，只在加载时解析一次
- 悬停提示生成时直接使用预存储的示例字符串
- 不会影响语言服务器的性能

## 未来改进

可能的改进方向：

1. **示例验证**: 在加载 Schema 时验证示例代码的语法正确性
2. **多个示例**: 支持为一个配置项提供多个示例
3. **交互式示例**: 允许用户直接插入示例代码到配置文件
4. **示例模板**: 支持示例代码中的占位符和变量替换

## 相关文件

- `spring-lsp/src/schema.rs`: Schema 数据模型定义
- `spring-lsp/src/toml_analyzer.rs`: TOML 分析器和悬停提示实现
- `spring-lsp/tests/toml_hover_test.rs`: 悬停提示单元测试

## 验证需求

该实现验证了以下需求：

- **Requirements 15.5**: 文档包含示例代码时，悬停提示应该包含格式化的示例代码
- **Property 64**: 文档示例显示 - 对于包含示例代码的文档，悬停提示应该包含格式化的示例代码

## 总结

示例代码显示功能成功实现，为开发者提供了更好的配置项文档体验。通过在悬停提示中显示格式化的示例代码，开发者可以快速了解配置项的正确用法，减少配置错误，提高开发效率。
