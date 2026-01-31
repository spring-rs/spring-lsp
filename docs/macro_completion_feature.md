# 宏参数补全功能

## 概述

宏参数补全功能为 spring-rs 框架的各种宏提供智能补全支持，帮助开发者快速正确地编写宏参数。

## 功能特性

### 1. Service 宏补全

为 `#[derive(Service)]` 宏的字段提供 `inject` 属性补全：

- `inject(component)` - 按类型注入组件
- `inject(component = "name")` - 按名称注入组件
- `inject(config)` - 注入配置

**示例**：
```rust
#[derive(Clone, Service)]
struct UserService {
    #[inject(component)]  // 补全提示
    db: ConnectPool,
    
    #[inject(component = "primary")]  // 补全提示
    primary_db: ConnectPool,
    
    #[inject(config)]  // 补全提示
    config: UserConfig,
}
```

### 2. Inject 宏补全

为 `#[inject]` 属性提供注入类型补全：

- `component` - 注入组件
- `config` - 注入配置

**示例**：
```rust
#[inject(component)]  // 补全提示：component, config
db: ConnectPool,
```

### 3. AutoConfig 宏补全

为 `#[auto_config]` 宏提供配置器类型补全：

- `WebConfigurator` - Web 路由配置器
- `JobConfigurator` - 任务调度配置器
- `StreamConfigurator` - 流处理配置器

**示例**：
```rust
#[auto_config(WebConfigurator)]  // 补全提示
#[tokio::main]
async fn main() {
    App::new().add_plugin(WebPlugin).run().await
}
```

### 4. Route 宏补全

为路由宏提供 HTTP 方法和路径参数补全：

**HTTP 方法补全**：
- `GET` - 获取资源
- `POST` - 创建资源
- `PUT` - 更新资源（完整）
- `DELETE` - 删除资源
- `PATCH` - 更新资源（部分）
- `HEAD` - 获取资源头信息
- `OPTIONS` - 获取支持的方法

**路径参数补全**：
- `{id}` - 路径参数占位符

**示例**：
```rust
#[get("/users/{id}")]  // 补全提示：GET, POST, PUT, DELETE, PATCH, {id}
async fn get_user(Path(id): Path<i64>) -> impl IntoResponse {
}
```

### 5. Job 宏补全

为任务调度宏提供 cron 表达式和延迟/频率值补全：

**Cron 表达式补全**：
- `0 0 * * * *` - 每小时执行
- `0 0 0 * * *` - 每天午夜执行
- `0 */5 * * * *` - 每 5 分钟执行

**延迟/频率值补全**：
- `5` - 5 秒
- `10` - 10 秒
- `60` - 60 秒（1 分钟）

**示例**：
```rust
#[cron("0 0 * * * *")]  // 补全提示：常用 cron 表达式
async fn hourly_job() {
}

#[fix_delay(5)]  // 补全提示：5, 10, 60
async fn delayed_job() {
}

#[fix_rate(10)]  // 补全提示：5, 10, 60
async fn periodic_job() {
}
```

## 实现细节

### CompletionEngine 结构

```rust
pub struct CompletionEngine {
    // 补全引擎字段
}

impl CompletionEngine {
    /// 为宏参数提供补全
    pub fn complete_macro(
        &self,
        macro_info: &SpringMacro,
        cursor_position: Option<&str>,
    ) -> Vec<CompletionItem>;
}
```

### 补全项结构

每个补全项包含：

- **label**: 补全项标签（显示给用户）
- **kind**: 补全项类型（PROPERTY、KEYWORD、CLASS、CONSTANT、VALUE、SNIPPET）
- **detail**: 简短描述
- **documentation**: 详细文档（Markdown 格式）
- **insert_text**: 插入文本
- **insert_text_format**: 插入文本格式（PLAIN_TEXT 或 SNIPPET）

### 补全项类型

- **PROPERTY**: Service 宏的 inject 属性
- **KEYWORD**: Inject 宏的注入类型
- **CLASS**: AutoConfig 宏的配置器类型
- **CONSTANT**: Route 宏的 HTTP 方法
- **VALUE**: Job 宏的延迟/频率值
- **SNIPPET**: 路径参数和 cron 表达式

## 测试覆盖

实现了 9 个单元测试，覆盖以下场景：

1. `test_complete_service_macro` - Service 宏补全
2. `test_complete_inject_macro` - Inject 宏补全
3. `test_complete_auto_config_macro` - AutoConfig 宏补全
4. `test_complete_route_macro` - Route 宏补全
5. `test_complete_job_macro_cron` - Cron 任务补全
6. `test_complete_job_macro_fix_delay` - FixDelay 任务补全
7. `test_complete_job_macro_fix_rate` - FixRate 任务补全
8. `test_completion_items_have_documentation` - 验证所有补全项都有文档
9. `test_completion_items_have_correct_kind` - 验证补全项类型正确

所有测试均通过。

## 使用示例

### 在 LSP 服务器中使用

```rust
use spring_lsp::completion::CompletionEngine;
use spring_lsp::macro_analyzer::{MacroAnalyzer, SpringMacro};

// 创建补全引擎
let engine = CompletionEngine::new();

// 解析 Rust 代码并提取宏
let analyzer = MacroAnalyzer::new();
let doc = analyzer.parse(uri, content)?;
let doc = analyzer.extract_macros(doc)?;

// 为每个宏提供补全
for macro_info in &doc.macros {
    let completions = engine.complete_macro(macro_info, None);
    // 将补全项发送给客户端
}
```

## 未来改进

1. **上下文感知补全**：根据光标位置提供更精确的补全
2. **动态组件名称补全**：从项目中提取已注册的组件名称
3. **路径参数类型推断**：根据处理器函数参数推断路径参数类型
4. **Cron 表达式验证**：实时验证 cron 表达式的正确性
5. **自定义配置器补全**：支持用户自定义的配置器类型

## 相关文档

- [需求文档](../../.kiro/specs/spring-lsp/requirements.md) - Requirement 7.5
- [设计文档](../../.kiro/specs/spring-lsp/design.md) - Property 35
- [任务列表](../../.kiro/specs/spring-lsp/tasks.md) - Task 15.1
