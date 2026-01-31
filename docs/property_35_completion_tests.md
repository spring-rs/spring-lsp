# Property 35: 宏参数补全属性测试

## 概述

Property 35 验证了补全引擎为所有 spring-rs 宏类型提供正确的参数补全功能。

**验证需求**: Requirements 7.5

**属性陈述**: *For any* 宏参数输入位置，补全引擎应该提供该宏支持的参数名称和值。

## 测试用例

### 1. Service 宏参数补全 (`prop_service_macro_completion_provides_inject_options`)

**测试目标**: 验证 Service 宏提供正确的 inject 属性补全

**验证内容**:
- 补全列表不为空
- 每个补全项包含必要信息（label、detail、documentation、insert_text、kind）
- 补全项类型为 PROPERTY
- 包含 `inject(component)` 补全
- 包含 `inject(config)` 补全

**迭代次数**: 100

### 2. Inject 宏参数补全 (`prop_inject_macro_completion_provides_type_options`)

**测试目标**: 验证 Inject 宏提供正确的注入类型补全

**验证内容**:
- 补全列表不为空
- 每个补全项包含必要信息
- 补全项类型为 KEYWORD
- 包含 `component` 补全
- 包含 `config` 补全

**迭代次数**: 100

### 3. AutoConfig 宏参数补全 (`prop_auto_config_macro_completion_provides_configurator_types`)

**测试目标**: 验证 AutoConfig 宏提供正确的配置器类型补全

**验证内容**:
- 补全列表不为空
- 每个补全项包含必要信息
- 补全项类型为 CLASS
- 包含常见的配置器类型（WebConfigurator、JobConfigurator、StreamConfigurator）

**迭代次数**: 100

### 4. Route 宏参数补全 (`prop_route_macro_completion_provides_http_methods_and_path_params`)

**测试目标**: 验证 Route 宏提供正确的 HTTP 方法和路径参数补全

**验证内容**:
- 补全列表不为空
- 每个补全项包含必要信息
- 包含 HTTP 方法补全（CONSTANT 类型）
- 包含常见的 HTTP 方法（GET、POST、PUT、DELETE）
- 包含路径参数补全（SNIPPET 类型）
- 路径参数补全包含模板（如 `{id}`）

**迭代次数**: 100

### 5. Job 宏参数补全 (`prop_job_macro_completion_provides_schedule_options`)

**测试目标**: 验证 Job 宏提供正确的调度选项补全

**验证内容**:
- 补全列表不为空
- 每个补全项包含必要信息
- 包含 cron 表达式补全（SNIPPET 类型）
- 包含延迟/频率值补全（VALUE 类型）
- Cron 表达式格式正确（至少 5 个字段）
- 延迟/频率值是数字

**迭代次数**: 100

### 6. 补全项文档完整性 (`prop_completion_items_have_complete_documentation`)

**测试目标**: 验证所有补全项都有完整的文档

**验证内容**:
- 所有宏类型都提供补全项
- 文档是 MarkupContent 类型
- 文档格式为 Markdown
- 文档内容不为空且包含有用信息

**迭代次数**: 100

### 7. 补全项插入文本正确性 (`prop_completion_items_have_valid_insert_text`)

**测试目标**: 验证所有补全项的插入文本正确

**验证内容**:
- 插入文本不为空
- Snippet 格式的插入文本包含占位符标记（`$`）
- 插入文本与 label 相关

**迭代次数**: 100

### 8. 补全项类型一致性 (`prop_completion_items_have_consistent_types`)

**测试目标**: 验证每种宏类型的补全项类型一致

**验证内容**:
- Service 宏: 所有补全项为 PROPERTY 类型
- Inject 宏: 所有补全项为 KEYWORD 类型
- AutoConfig 宏: 所有补全项为 CLASS 类型
- Route 宏: 补全项为 CONSTANT 或 SNIPPET 类型
- Job 宏: 补全项为 SNIPPET 或 VALUE 类型

**迭代次数**: 100

### 9. 补全引擎不应崩溃 (`prop_completion_engine_does_not_crash`)

**测试目标**: 验证补全引擎在任何输入下都不会崩溃

**验证内容**:
- 解析和提取宏成功
- 对每个宏调用 `complete_macro` 不会 panic
- 执行完成不会崩溃

**迭代次数**: 100

### 10. 补全项数量合理性 (`prop_completion_items_count_is_reasonable`)

**测试目标**: 验证补全项数量在合理范围内

**验证内容**:
- 补全项数量在 1-100 之间
- Service 宏: 至少 3 个补全项
- Inject 宏: 至少 2 个补全项
- AutoConfig 宏: 至少 3 个补全项
- Route 宏: 至少 8 个补全项
- Job 宏: 至少 6 个补全项

**迭代次数**: 100

## 测试结果

所有 10 个属性测试均通过，每个测试运行 100 次迭代。

```
test prop_service_macro_completion_provides_inject_options ... ok
test prop_inject_macro_completion_provides_type_options ... ok
test prop_auto_config_macro_completion_provides_configurator_types ... ok
test prop_route_macro_completion_provides_http_methods_and_path_params ... ok
test prop_job_macro_completion_provides_schedule_options ... ok
test prop_completion_items_have_complete_documentation ... ok
test prop_completion_items_have_valid_insert_text ... ok
test prop_completion_items_have_consistent_types ... ok
test prop_completion_engine_does_not_crash ... ok
test prop_completion_items_count_is_reasonable ... ok
```

## 覆盖的补全类型

### Service 宏补全
- `inject(component)` - 注入组件
- `inject(component = "name")` - 注入指定名称的组件
- `inject(config)` - 注入配置

### Inject 宏补全
- `component` - 组件注入类型
- `config` - 配置注入类型

### AutoConfig 宏补全
- `WebConfigurator` - Web 路由配置器
- `JobConfigurator` - 任务调度配置器
- `StreamConfigurator` - 流处理配置器

### Route 宏补全
- HTTP 方法: `GET`, `POST`, `PUT`, `DELETE`, `PATCH`, `HEAD`, `OPTIONS`
- 路径参数模板: `{id}`

### Job 宏补全
- Cron 表达式:
  - `0 0 * * * *` - 每小时执行
  - `0 0 0 * * *` - 每天午夜执行
  - `0 */5 * * * *` - 每 5 分钟执行
- 延迟/频率值: `5`, `10`, `60`

## 补全项结构验证

每个补全项都包含以下必要信息：

1. **label**: 补全项的显示文本（非空）
2. **detail**: 补全项的简短描述（非空）
3. **documentation**: 补全项的详细文档（Markdown 格式，非空）
4. **insert_text**: 插入到编辑器的文本（非空）
5. **kind**: 补全项的类型（PROPERTY、KEYWORD、CLASS、CONSTANT、VALUE、SNIPPET）
6. **insert_text_format**: 插入文本的格式（PlainText 或 Snippet）

## 测试策略

### 数据生成
使用 proptest 生成随机的 Rust 代码，包括：
- 简单的结构体定义
- 带有 `#[derive(Service)]` 的结构体
- 带有 `#[inject]` 属性的字段
- 路由宏标注的函数
- 任务宏标注的函数
- 复杂的混合代码

### 验证方法
1. **结构验证**: 检查补全项的结构完整性
2. **类型验证**: 检查补全项的类型正确性
3. **内容验证**: 检查补全项的内容合理性
4. **数量验证**: 检查补全项的数量在合理范围内
5. **稳定性验证**: 检查补全引擎不会崩溃

### 迭代次数
每个属性测试运行 100 次迭代，确保在各种随机输入下都能正确工作。

## 与单元测试的关系

属性测试和单元测试是互补的：

- **单元测试** (`src/completion/tests.rs`): 验证具体的示例和边缘情况
- **属性测试** (`tests/macro_analyzer_property_test.rs`): 验证通用属性在所有输入下的正确性

两种测试方法共同确保补全功能的全面覆盖和正确性。

## 结论

Property 35 的所有测试均通过，验证了补全引擎能够为所有 spring-rs 宏类型提供正确、完整、一致的参数补全。补全项包含必要的信息，类型正确，数量合理，且引擎在任何输入下都不会崩溃。
