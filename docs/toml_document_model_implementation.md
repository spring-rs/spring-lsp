# TOML 文档模型实现文档

## 概述

本文档记录了 spring-lsp 项目中 TOML 文档模型的实现，对应任务 6.1。

## 实现的数据结构

### 1. TomlDocument

主要的 TOML 文档结构，包含：
- `root: taplo::dom::Node` - taplo 库的 DOM 根节点
- `env_vars: Vec<EnvVarReference>` - 提取的环境变量引用列表
- `config_sections: HashMap<String, ConfigSection>` - 配置节映射（键为配置前缀）

### 2. EnvVarReference

表示环境变量插值引用，如 `${HOST:localhost}` 或 `${PORT}`：
- `name: String` - 环境变量名称
- `default: Option<String>` - 默认值（可选）
- `range: Range` - 在文档中的位置范围

**特性**：
- 实现了 `PartialEq` 和 `Eq`，支持相等性比较
- 支持带默认值和不带默认值两种形式

### 3. ConfigSection

表示 TOML 配置节，如 `[web]` 或 `[redis]`：
- `prefix: String` - 配置前缀（节名称）
- `properties: HashMap<String, ConfigProperty>` - 配置属性映射
- `range: Range` - 在文档中的位置范围

### 4. ConfigProperty

表示配置节中的单个属性，如 `host = "localhost"`：
- `key: String` - 属性键
- `value: ConfigValue` - 属性值
- `range: Range` - 在文档中的位置范围

### 5. ConfigValue

表示配置属性的值，支持多种类型：
- `String(String)` - 字符串值
- `Integer(i64)` - 整数值
- `Float(f64)` - 浮点数值
- `Boolean(bool)` - 布尔值
- `Array(Vec<ConfigValue>)` - 数组值（支持嵌套）
- `Table(HashMap<String, ConfigValue>)` - 表值（支持嵌套）

**特性**：
- 实现了 `PartialEq`，支持值的相等性比较
- 支持递归嵌套结构（数组和表可以包含任意类型的值）

## 设计决策

### 1. 使用 taplo 的 DOM API

选择使用 taplo 库的 DOM API 而不是直接使用 serde 反序列化，原因：
- **保留位置信息**：DOM API 保留了所有节点的源代码位置，这对于 LSP 功能（如诊断、补全、悬停）至关重要
- **错误恢复**：即使 TOML 文件有语法错误，DOM API 也能部分解析，提供更好的用户体验
- **灵活性**：可以遍历和查询 TOML 文档结构，提取特定信息（如环境变量引用）

### 2. 分离的数据结构

将 TOML 文档模型与 Schema 模型分离：
- `ConfigValue` 用于表示 TOML 文档中的实际值
- `schema::Value` 用于表示 Schema 中的默认值
- 这种分离使得两个模块可以独立演化，避免耦合

### 3. 位置信息

所有主要的数据结构都包含 `range: Range` 字段：
- 用于生成准确的诊断信息
- 用于实现跳转和导航功能
- 使用 LSP 标准的 `Range` 类型，确保与编辑器的兼容性

### 4. 环境变量引用的独立提取

将环境变量引用作为独立的列表存储：
- 便于快速查找所有环境变量引用
- 支持环境变量的补全和验证
- 不影响配置节的结构

## 测试覆盖

实现了 10 个单元测试，覆盖以下场景：

1. **基本功能测试**：
   - `test_env_var_reference_creation` - 创建带默认值的环境变量引用
   - `test_env_var_reference_without_default` - 创建不带默认值的环境变量引用
   - `test_config_property_creation` - 创建配置属性
   - `test_config_section_creation` - 创建配置节
   - `test_toml_document_creation` - 创建完整的 TOML 文档

2. **类型测试**：
   - `test_config_value_types` - 测试所有配置值类型
   - `test_nested_config_values` - 测试嵌套的配置值

3. **边缘情况测试**：
   - `test_empty_toml_document` - 测试空 TOML 文档

4. **相等性测试**：
   - `test_config_value_equality` - 测试配置值的相等性比较
   - `test_env_var_reference_equality` - 测试环境变量引用的相等性比较

所有测试均通过，确保数据结构的正确性。

## 与设计文档的对应关系

本实现完全符合设计文档中的 "Data Models - Configuration Models" 部分：

- ✅ `EnvVarReference` - 环境变量引用
- ✅ `ConfigSection` - 配置节
- ✅ `ConfigProperty` - 配置属性
- ✅ `ConfigValue` - 配置值（对应设计文档中的 `Value`）
- ✅ `TomlDocument` - TOML 文档（包含 taplo DOM 根节点）

## 满足的需求

本实现满足以下需求：

- **Requirements 2.1**: 支持解析 TOML 文件的数据结构
- **Requirements 2.3**: 支持识别环境变量插值的数据结构

## 后续工作

下一步将实现 `TomlAnalyzer` 的解析方法（任务 6.2），使用这些数据结构来：
1. 解析 TOML 文件内容
2. 提取环境变量引用
3. 提取配置节和属性
4. 处理语法错误

## 文件位置

- **实现文件**: `spring-lsp/src/toml_analyzer.rs`
- **测试文件**: `spring-lsp/tests/toml_document_model_test.rs`
- **文档文件**: `spring-lsp/docs/toml_document_model_implementation.md`

## 编译和测试

```bash
# 编译检查
cargo check

# 运行测试
cargo test --test toml_document_model_test

# 查看测试输出
cargo test --test toml_document_model_test -- --nocapture
```

所有测试均通过，代码编译无警告。
