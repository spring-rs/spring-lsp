# TOML 解析器属性测试文档

## 概述

本文档描述了为 spring-lsp 的 TOML 解析器实现的属性测试（Property-Based Tests）。属性测试使用 `proptest` 库，通过随机生成的输入验证系统的通用属性。

## 测试配置

- **测试框架**: proptest 1.4
- **迭代次数**: 每个属性测试运行 100 次
- **测试文件**: `tests/toml_analyzer_property_test.rs`

## 实现的属性测试

### Property 6: TOML 解析成功性

**验证需求**: Requirements 2.1

**属性描述**: For any 语法正确的 TOML 文件，解析器应该成功解析并返回文档结构。

**测试策略**:
- 生成 0-10 个有效的 TOML 配置节
- 每个配置节包含 0-5 个属性
- 属性键使用小写字母和下划线
- 属性值使用字母、数字和常见符号

**测试用例**:
1. `prop_parse_valid_toml_succeeds`: 测试多个配置节的解析
2. `prop_parse_empty_toml_succeeds`: 测试空文件的解析

**验证点**:
- 解析应该成功（`result.is_ok()`）
- 配置节数量应该合理
- 空文件应该产生空的配置节列表

### Property 7: TOML 错误报告

**验证需求**: Requirements 2.2

**属性描述**: For any 包含语法错误的 TOML 文件，解析器应该返回包含错误位置和描述的诊断信息。

**测试策略**:
- 生成各种无效的 TOML 内容：
  - 未闭合的节：`[invalid`
  - 未完成的赋值：`[section\nkey = `
  - 未闭合的字符串：`key = "unclosed`
  - 未引用的字符串值：`[section]\nkey = value`
  - 缺少值：`[section]\nkey =`
  - 缺少键：`= value`

**测试用例**:
- `prop_parse_invalid_toml_returns_error`

**验证点**:
- 解析应该失败（`result.is_err()`）
- 错误消息应该包含 "TOML 语法错误"
- 错误消息应该非空

### Property 8: 环境变量识别

**验证需求**: Requirements 2.3

**属性描述**: For any 包含 `${VAR:default}` 格式的 TOML 文件，解析器应该正确提取变量名 `VAR` 和默认值 `default`。

**测试策略**:
- 生成有效的环境变量名（大写字母和下划线）
- 生成可选的默认值
- 构建包含环境变量插值的 TOML 配置

**测试用例**:
1. `prop_env_var_extraction_with_default`: 测试单个环境变量的提取
2. `prop_multiple_env_vars_extraction`: 测试多个环境变量的提取

**验证点**:
- 解析应该成功
- 应该识别到正确数量的环境变量
- 变量名应该正确提取
- 默认值应该正确提取（如果存在）

### Property 9: 多环境配置支持

**验证需求**: Requirements 2.4

**属性描述**: For any 环境配置文件（如 `app-dev.toml`、`app-prod.toml`），解析器应该能够成功解析。

**测试策略**:
- 生成不同环境的配置文件名：
  - `app.toml`
  - `app-dev.toml`
  - `app-test.toml`
  - `app-prod.toml`
  - `app-staging.toml`
- 使用相同的配置结构测试不同环境

**测试用例**:
1. `prop_multi_env_config_support`: 测试不同环境配置文件的解析
2. `prop_same_structure_across_envs`: 测试不同环境使用相同配置结构

**验证点**:
- 所有环境的配置文件都应该成功解析
- 不同环境应该能够使用相同的配置节结构
- 配置节名称应该在不同环境中保持一致

## 额外的属性测试

除了设计文档中定义的 4 个核心属性，我们还实现了 3 个额外的属性测试来增强测试覆盖：

### 解析幂等性

**属性描述**: 多次解析相同的 TOML 应该产生相同的结果。

**测试用例**: `prop_parse_idempotent`

**验证点**:
- 两次解析的结果应该一致（都成功或都失败）
- 配置节数量应该相同
- 环境变量数量应该相同

### 配置节数量合理性

**属性描述**: 解析后的配置节数量应该等于输入的节数量。

**测试用例**: `prop_section_count_reasonable`

**验证点**:
- 配置节数量应该等于输入的节数量
- 所有节名称都应该被识别
- 每个节都应该在结果中存在

## 测试策略生成器

### 有效 TOML 生成器

```rust
fn valid_section_name() -> impl Strategy<Value = String>
fn valid_property_key() -> impl Strategy<Value = String>
fn valid_string_value() -> impl Strategy<Value = String>
fn valid_env_var_name() -> impl Strategy<Value = String>
fn valid_toml_section() -> impl Strategy<Value = String>
```

### 环境变量生成器

```rust
fn toml_with_env_var() -> impl Strategy<Value = (String, String, Option<String>)>
```

### 无效 TOML 生成器

```rust
fn invalid_toml() -> impl Strategy<Value = String>
```

### 多环境配置生成器

```rust
fn multi_env_config_name() -> impl Strategy<Value = String>
```

## 测试结果

所有 9 个属性测试都通过了 100 次迭代：

```
test prop_parse_empty_toml_succeeds ... ok
test prop_env_var_extraction_with_default ... ok
test prop_multiple_env_vars_extraction ... ok
test prop_parse_invalid_toml_returns_error ... ok
test prop_same_structure_across_envs ... ok
test prop_section_count_reasonable ... ok
test prop_multi_env_config_support ... ok
test prop_parse_idempotent ... ok
test prop_parse_valid_toml_succeeds ... ok
```

## 与单元测试的关系

属性测试和单元测试是互补的：

- **单元测试** (`tests/toml_analyzer_test.rs`): 验证具体的示例和边缘情况
  - 测试空 TOML 文件
  - 测试特定的语法错误
  - 测试特定的环境变量格式
  - 测试特定的数据类型（数组、布尔值、浮点数）

- **属性测试** (`tests/toml_analyzer_property_test.rs`): 验证通用属性
  - 测试任意有效的 TOML 都能成功解析
  - 测试任意无效的 TOML 都会报错
  - 测试任意环境变量格式都能正确识别
  - 测试解析的幂等性和一致性

## 发现的问题

在实现属性测试的过程中，我们发现了一个问题：

**问题**: 初始的无效 TOML 生成器包含了 `"[[array]"`，但这实际上是有效的 TOML（数组表）。

**解决方案**: 更新了无效 TOML 生成器，移除了实际上有效的 TOML 模式，只保留真正无效的语法。

## 运行测试

```bash
# 运行所有属性测试
cargo test --test toml_analyzer_property_test

# 运行特定的属性测试
cargo test --test toml_analyzer_property_test prop_parse_valid_toml_succeeds

# 运行所有 TOML 相关测试（单元测试 + 属性测试）
cargo test --lib --test toml_analyzer_test --test toml_analyzer_property_test
```

## 未来改进

1. **增加迭代次数**: 在 CI 环境中可以运行更多迭代（如 1000 次）
2. **更复杂的生成器**: 生成嵌套的 TOML 表和更复杂的数据结构
3. **性能属性**: 添加性能相关的属性测试（如解析时间应该与输入大小成线性关系）
4. **错误恢复**: 测试部分无效的 TOML 是否能够部分解析

## 参考

- [proptest 文档](https://docs.rs/proptest/)
- [TOML 规范](https://toml.io/)
- [spring-lsp 设计文档](../design.md)
- [spring-lsp 需求文档](../requirements.md)
