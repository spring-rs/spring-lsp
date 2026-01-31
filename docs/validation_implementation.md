# TOML 配置验证实现文档

## 概述

本文档描述了 spring-lsp 项目中 TOML 配置验证功能的实现细节。

## 实现的功能

### 1. 配置项定义检查

验证配置文件中的配置节和配置项是否在 Schema 中定义。

**实现位置**: `TomlAnalyzer::validate()`

**验证逻辑**:
- 检查配置节（如 `[web]`）是否在 Schema 的 `plugins` 中定义
- 检查配置项（如 `host`、`port`）是否在对应插件的 `properties` 中定义
- 未定义的配置节或配置项会生成 ERROR 级别的诊断

**示例**:
```toml
[unknown_section]  # 错误：未在 Schema 中定义
key = "value"

[web]
unknown_key = "value"  # 错误：未在 Schema 中定义
```

### 2. 类型验证

验证配置值的类型是否与 Schema 中定义的类型匹配。

**实现位置**: `TomlAnalyzer::validate_property_type()`

**支持的类型**:
- String（字符串）
- Integer（整数）
- Float（浮点数）
- Boolean（布尔值）
- Array（数组）
- Table（对象/表）

**验证逻辑**:
- 将配置值的实际类型与 Schema 中定义的类型进行匹配
- 类型不匹配时生成 ERROR 级别的诊断，包含期望类型和实际类型

**示例**:
```toml
[web]
port = "not_a_number"  # 错误：期望整数，实际字符串
```

### 3. 必需项检查

检查 Schema 中标记为必需的配置项是否存在。

**实现位置**: `TomlAnalyzer::validate_required_properties()`

**验证逻辑**:
- 遍历插件 Schema 中的所有属性
- 检查 `required` 字段为 `true` 的属性是否在配置节中存在
- 缺失的必需配置项会生成 WARNING 级别的诊断

**示例**:
```toml
[web]
host = "localhost"
# 警告：缺少必需的配置项 'port'
```

### 4. 废弃项检查

检查是否使用了 Schema 中标记为废弃的配置项。

**实现位置**: `TomlAnalyzer::validate_section()`

**验证逻辑**:
- 检查配置项的 `deprecated` 字段
- 如果使用了废弃的配置项，生成 WARNING 级别的诊断
- 诊断消息包含废弃原因和替代建议

**示例**:
```toml
[web]
old_config = "value"  # 警告：配置项 'old_config' 已废弃: 请使用 new_config 代替
```

### 5. 环境变量语法验证

验证环境变量插值表达式的语法是否正确。

**实现位置**: `TomlAnalyzer::validate_env_var_syntax()`

**验证规则**:
1. 环境变量名不能为空
2. 环境变量名应该符合命名规范（大写字母、数字、下划线）

**验证逻辑**:
- 检查环境变量名是否为空，如果为空生成 ERROR 级别的诊断
- 检查环境变量名是否只包含大写字母、数字和下划线
- 不符合命名规范的变量名生成 WARNING 级别的诊断

**示例**:
```toml
[web]
host = "${}"  # 错误：环境变量名不能为空
url = "${invalid-name}"  # 警告：不符合命名规范，建议使用大写字母、数字和下划线
```

### 6. 值范围验证

验证配置值是否在 Schema 定义的允许范围内。

**实现位置**: `TomlAnalyzer::validate_property_range()`

**支持的范围约束**:

#### 字符串类型
- `min_length`: 最小长度
- `max_length`: 最大长度
- `enum_values`: 枚举值列表

**示例**:
```toml
[web]
host = ""  # 错误：长度小于最小长度 1
mode = "invalid"  # 错误：不在允许的枚举值中 ["dev", "prod"]
```

#### 整数类型
- `min`: 最小值
- `max`: 最大值

**示例**:
```toml
[web]
port = 0  # 错误：值小于最小值 1
port = 70000  # 错误：值超过最大值 65535
```

#### 浮点数类型
- `min`: 最小值
- `max`: 最大值

**示例**:
```toml
[web]
timeout = -1.0  # 错误：值小于最小值 0.0
```

## 诊断信息格式

所有诊断信息都遵循 LSP 标准格式：

```rust
Diagnostic {
    range: Range,                    // 错误位置
    severity: DiagnosticSeverity,    // 严重程度（ERROR/WARNING）
    code: String,                     // 错误代码
    message: String,                  // 错误消息
    source: "spring-lsp",            // 来源
    ..Default::default()
}
```

### 错误代码列表

| 错误代码 | 严重程度 | 描述 |
|---------|---------|------|
| `undefined-section` | ERROR | 配置节未在 Schema 中定义 |
| `undefined-property` | ERROR | 配置项未在 Schema 中定义 |
| `type-mismatch` | ERROR | 配置值类型不匹配 |
| `missing-required-property` | WARNING | 缺少必需的配置项 |
| `deprecated-property` | WARNING | 使用了废弃的配置项 |
| `empty-var-name` | ERROR | 环境变量名为空 |
| `invalid-var-name` | WARNING | 环境变量名不符合命名规范 |
| `invalid-enum-value` | ERROR | 枚举值无效 |
| `string-too-short` | ERROR | 字符串长度小于最小长度 |
| `string-too-long` | ERROR | 字符串长度超过最大长度 |
| `value-too-small` | ERROR | 值小于最小值 |
| `value-too-large` | ERROR | 值超过最大值 |

## 使用示例

```rust
use spring_lsp::schema::SchemaProvider;
use spring_lsp::toml_analyzer::TomlAnalyzer;

// 创建 Schema 提供者
let schema_provider = SchemaProvider::default();

// 创建 TOML 分析器
let analyzer = TomlAnalyzer::new(schema_provider);

// 解析 TOML 文件
let toml_content = r#"
[web]
host = "localhost"
port = 8080
"#;

let doc = analyzer.parse(toml_content).unwrap();

// 验证配置
let diagnostics = analyzer.validate(&doc);

// 处理诊断信息
for diagnostic in diagnostics {
    println!("{:?}: {}", diagnostic.severity, diagnostic.message);
}
```

## 测试覆盖

### 单元测试

实现了 14 个单元测试，覆盖所有验证功能：

1. `test_validate_undefined_section` - 未定义的配置节
2. `test_validate_undefined_property` - 未定义的配置项
3. `test_validate_type_mismatch` - 类型不匹配
4. `test_validate_missing_required_property` - 缺少必需配置项
5. `test_validate_deprecated_property` - 废弃的配置项
6. `test_validate_env_var_empty_name` - 环境变量名为空
7. `test_validate_env_var_invalid_name` - 环境变量名不符合规范
8. `test_validate_integer_out_of_range_min` - 整数小于最小值
9. `test_validate_integer_out_of_range_max` - 整数超过最大值
10. `test_validate_string_too_short` - 字符串长度不足
11. `test_validate_invalid_enum_value` - 无效的枚举值
12. `test_validate_valid_config` - 有效的配置（无错误）
13. `test_validate_valid_env_var` - 有效的环境变量
14. `test_validate_multiple_errors` - 多个错误

所有测试均通过。

### 属性测试

属性测试将在任务 7.2 中实现，验证以下属性：

- Property 19: 配置项定义验证
- Property 20: 配置类型验证
- Property 21: 必需配置项检查
- Property 22: 废弃配置项警告
- Property 23: 环境变量语法验证
- Property 24: 配置值范围验证

## 架构设计

### 依赖关系

```
TomlAnalyzer
    ├── SchemaProvider (提供 Schema 查询)
    │   └── ConfigSchema
    │       └── PluginSchema
    │           └── PropertySchema
    │               └── TypeInfo
    └── TomlDocument (解析结果)
        ├── ConfigSection
        │   └── ConfigProperty
        │       └── ConfigValue
        └── EnvVarReference
```

### 验证流程

```
1. 解析 TOML 文件 -> TomlDocument
2. 验证环境变量语法
3. 遍历配置节
   3.1 检查配置节是否在 Schema 中定义
   3.2 验证配置节中的属性
       3.2.1 检查属性是否在 Schema 中定义
       3.2.2 检查属性是否废弃
       3.2.3 验证属性类型
       3.2.4 验证属性值范围
   3.3 检查必需的配置项是否存在
4. 返回诊断信息列表
```

## 性能考虑

1. **Schema 缓存**: SchemaProvider 使用 DashMap 缓存查询结果，避免重复查找
2. **并发安全**: 使用 DashMap 提供无锁并发访问
3. **增量验证**: 只验证修改的文档，不影响其他文档
4. **早期返回**: 在发现错误时继续验证其他项，收集所有错误

## 未来改进

1. **自定义验证规则**: 支持用户定义的验证规则
2. **快速修复**: 为常见错误提供自动修复建议
3. **更详细的错误信息**: 提供更多上下文和示例
4. **性能优化**: 使用并行验证提高大文件的验证速度
5. **增量验证**: 只验证修改的部分，而不是整个文档

## 相关文件

- 实现: `spring-lsp/src/toml_analyzer.rs`
- Schema: `spring-lsp/src/schema.rs`
- 单元测试: `spring-lsp/tests/toml_validation_test.rs`
- 设计文档: `.kiro/specs/spring-lsp/design.md`
- 需求文档: `.kiro/specs/spring-lsp/requirements.md`
