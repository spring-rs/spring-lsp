# TOML 配置验证属性测试文档

## 概述

本文档描述了为 spring-lsp 的 TOML 配置验证功能实现的属性测试（Property-Based Tests）。属性测试使用 `proptest` 库，通过随机生成的输入验证配置验证功能的通用属性。

## 测试配置

- **测试框架**: proptest 1.4
- **迭代次数**: 每个属性测试运行 100 次
- **测试文件**: `tests/toml_validation_property_test.rs`
- **测试数量**: 18 个属性测试

## 实现的属性测试

### Property 19: 配置项定义验证

**验证需求**: Requirements 5.1

**属性描述**: For any 配置文件中的配置项，如果该配置项不在 Schema 中定义，验证器应该生成错误诊断。

**测试用例**:

1. **`prop_undefined_property_generates_error`**
   - 生成包含已定义和未定义配置项的 Schema
   - 创建使用未定义配置项的 TOML
   - 验证生成了 `undefined-property` 错误诊断

2. **`prop_undefined_section_generates_error`**
   - 生成只包含特定配置节的 Schema
   - 创建使用未定义配置节的 TOML
   - 验证生成了 `undefined-section` 错误诊断

**测试策略**:
- 使用 `prop_assume!` 确保已定义和未定义的键/节不同
- 验证错误诊断包含正确的错误代码和消息

### Property 20: 配置类型验证

**验证需求**: Requirements 5.2

**属性描述**: For any 配置项的值，如果其类型与 Schema 中定义的类型不匹配，验证器应该生成类型错误诊断。

**测试用例**:

1. **`prop_type_mismatch_generates_error`**
   - 创建期望整数类型的 Schema
   - 提供字符串值
   - 验证生成了 `type-mismatch` 错误诊断

2. **`prop_type_mismatch_integer_for_string`**
   - 创建期望字符串类型的 Schema
   - 提供整数值
   - 验证生成了 `type-mismatch` 错误诊断

**测试策略**:
- 测试不同类型组合的不匹配情况
- 验证错误消息包含期望类型和实际类型

### Property 21: 必需配置项检查

**验证需求**: Requirements 5.3

**属性描述**: For any 在 Schema 中标记为必需的配置项，如果在配置文件中缺失，验证器应该生成警告诊断。

**测试用例**:

1. **`prop_missing_required_property_generates_warning`**
   - 创建包含必需和可选配置项的 Schema
   - 只提供可选配置项
   - 验证生成了 `missing-required-property` 警告诊断

2. **`prop_required_property_present_no_warning`**
   - 创建包含必需配置项的 Schema
   - 提供必需配置项
   - 验证没有生成缺失警告

**测试策略**:
- 使用 `required: true` 标记必需配置项
- 验证警告级别为 `WARNING`
- 测试正向和负向情况

### Property 22: 废弃配置项警告

**验证需求**: Requirements 5.4

**属性描述**: For any 在 Schema 中标记为废弃的配置项，如果在配置文件中使用，验证器应该生成废弃警告并提供替代建议。

**测试用例**:

1. **`prop_deprecated_property_generates_warning`**
   - 创建包含废弃配置项的 Schema（带废弃消息）
   - 使用废弃配置项
   - 验证生成了 `deprecated-property` 警告诊断
   - 验证警告消息包含废弃原因

**测试策略**:
- 使用 `deprecated: Some(message)` 标记废弃配置项
- 生成随机的废弃消息
- 验证警告消息包含废弃原因

### Property 23: 环境变量语法验证

**验证需求**: Requirements 5.5

**属性描述**: For any 环境变量插值表达式，如果语法不符合规范，验证器应该生成错误或警告诊断。

**测试用例**:

1. **`prop_empty_env_var_name_generates_error`**
   - 创建包含空环境变量名的 TOML（`${}`）
   - 验证生成了 `empty-var-name` 错误诊断

2. **`prop_invalid_env_var_name_generates_warning`**
   - 创建包含无效环境变量名的 TOML（如 `${invalid-name}`）
   - 验证生成了 `invalid-var-name` 警告诊断

3. **`prop_valid_env_var_name_no_error`**
   - 创建包含有效环境变量名的 TOML（大写字母、数字、下划线）
   - 验证没有生成环境变量相关的错误或警告

**测试策略**:
- 生成有效的环境变量名（`[A-Z][A-Z0-9_]*`）
- 生成无效的环境变量名（包含小写字母、连字符等）
- 验证空变量名生成错误，无效命名生成警告

### Property 24: 配置值范围验证

**验证需求**: Requirements 5.6

**属性描述**: For any 配置项的值，如果超出 Schema 中定义的允许范围，验证器应该生成范围错误诊断。

**测试用例**:

#### 整数范围验证

1. **`prop_integer_below_min_generates_error`**
   - 创建有最小值限制的 Schema
   - 提供小于最小值的整数
   - 验证生成了 `value-too-small` 错误诊断

2. **`prop_integer_above_max_generates_error`**
   - 创建有最大值限制的 Schema
   - 提供超过最大值的整数
   - 验证生成了 `value-too-large` 错误诊断

3. **`prop_integer_within_range_no_error`**
   - 创建有范围限制的 Schema
   - 提供范围内的整数
   - 验证没有生成范围错误

#### 字符串长度验证

4. **`prop_string_below_min_length_generates_error`**
   - 创建有最小长度限制的 Schema
   - 提供长度不足的字符串
   - 验证生成了 `string-too-short` 错误诊断

5. **`prop_string_above_max_length_generates_error`**
   - 创建有最大长度限制的 Schema
   - 提供长度超限的字符串
   - 验证生成了 `string-too-long` 错误诊断

#### 枚举值验证

6. **`prop_invalid_enum_value_generates_error`**
   - 创建有枚举值限制的 Schema
   - 提供不在枚举列表中的值
   - 验证生成了 `invalid-enum-value` 错误诊断

7. **`prop_valid_enum_value_no_error`**
   - 创建有枚举值限制的 Schema
   - 提供枚举列表中的值
   - 验证没有生成枚举值错误

#### 浮点数范围验证

8. **`prop_float_out_of_range_generates_error`**
   - 创建有范围限制的 Schema
   - 提供超出范围的浮点数
   - 验证生成了 `value-too-small` 错误诊断

**测试策略**:
- 使用随机生成的范围边界值
- 测试边界内外的值
- 验证正向和负向情况

## 测试策略生成器

### 基础生成器

```rust
/// 生成有效的配置节名称
fn valid_section_name() -> impl Strategy<Value = String>

/// 生成有效的配置键名
fn valid_property_key() -> impl Strategy<Value = String>

/// 生成有效的字符串值
fn valid_string_value() -> impl Strategy<Value = String>

/// 生成有效的环境变量名
fn valid_env_var_name() -> impl Strategy<Value = String>

/// 生成无效的环境变量名
fn invalid_env_var_name() -> impl Strategy<Value = String>

/// 生成整数值
fn integer_value() -> impl Strategy<Value = i64>

/// 生成浮点数值（未使用）
fn float_value() -> impl Strategy<Value = f64>
```

### Schema 构建辅助函数

```rust
/// 创建包含指定插件的 Schema
fn create_schema_with_plugin(
    plugin_name: &str,
    properties: HashMap<String, PropertySchema>,
) -> ConfigSchema

/// 创建分析器
fn create_analyzer_with_schema(schema: ConfigSchema) -> TomlAnalyzer
```

## 测试结果

所有 18 个属性测试都通过了 100 次迭代：

```
test prop_deprecated_property_generates_warning ... ok
test prop_empty_env_var_name_generates_error ... ok
test prop_float_out_of_range_generates_error ... ok
test prop_integer_above_max_generates_error ... ok
test prop_integer_below_min_generates_error ... ok
test prop_integer_within_range_no_error ... ok
test prop_invalid_enum_value_generates_error ... ok
test prop_invalid_env_var_name_generates_warning ... ok
test prop_missing_required_property_generates_warning ... ok
test prop_required_property_present_no_warning ... ok
test prop_string_above_max_length_generates_error ... ok
test prop_string_below_min_length_generates_error ... ok
test prop_type_mismatch_generates_error ... ok
test prop_type_mismatch_integer_for_string ... ok
test prop_undefined_property_generates_error ... ok
test prop_undefined_section_generates_error ... ok
test prop_valid_enum_value_no_error ... ok
test prop_valid_env_var_name_no_error ... ok
```

## 测试覆盖

### 验证的属性

- ✅ Property 19: 配置项定义验证（2 个测试）
- ✅ Property 20: 配置类型验证（2 个测试）
- ✅ Property 21: 必需配置项检查（2 个测试）
- ✅ Property 22: 废弃配置项警告（1 个测试）
- ✅ Property 23: 环境变量语法验证（3 个测试）
- ✅ Property 24: 配置值范围验证（8 个测试）

### 验证的需求

- ✅ Requirements 5.1: 配置项定义检查
- ✅ Requirements 5.2: 类型验证
- ✅ Requirements 5.3: 必需项检查
- ✅ Requirements 5.4: 废弃项检查
- ✅ Requirements 5.5: 环境变量语法验证
- ✅ Requirements 5.6: 值范围验证

## 与单元测试的关系

属性测试和单元测试是互补的：

- **单元测试** (`tests/toml_validation_test.rs`): 验证具体的示例和边缘情况
  - 测试特定的错误场景
  - 测试特定的配置值
  - 测试多个错误的组合

- **属性测试** (`tests/toml_validation_property_test.rs`): 验证通用属性
  - 测试任意配置项的定义验证
  - 测试任意类型的类型验证
  - 测试任意范围的值范围验证
  - 测试验证逻辑的一致性

## 发现的问题

在实现属性测试的过程中，没有发现实现中的问题。所有测试都按预期通过，验证了配置验证功能的正确性。

## 运行测试

```bash
# 运行所有配置验证属性测试
cargo test --test toml_validation_property_test

# 运行特定的属性测试
cargo test --test toml_validation_property_test prop_undefined_property_generates_error

# 运行所有配置验证测试（单元测试 + 属性测试）
cargo test --test toml_validation_test --test toml_validation_property_test
```

## 性能考虑

- 每个属性测试运行 100 次迭代
- 总测试时间约 0.73 秒
- 平均每个测试约 40 毫秒
- 测试性能良好，适合在 CI 环境中运行

## 未来改进

1. **增加迭代次数**: 在 CI 环境中可以运行更多迭代（如 1000 次）
2. **更复杂的生成器**: 生成嵌套的配置结构和更复杂的 Schema
3. **组合验证**: 测试多个验证错误同时出现的情况
4. **性能属性**: 添加性能相关的属性测试（如验证时间应该与配置项数量成线性关系）
5. **错误恢复**: 测试部分无效的配置是否能够继续验证其他部分

## 参考

- [proptest 文档](https://docs.rs/proptest/)
- [spring-lsp 设计文档](../../.kiro/specs/spring-lsp/design.md)
- [spring-lsp 需求文档](../../.kiro/specs/spring-lsp/requirements.md)
- [配置验证实现文档](./validation_implementation.md)
- [单元测试](../tests/toml_validation_test.rs)

