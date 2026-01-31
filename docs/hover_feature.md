# 宏悬停提示功能

## 概述

宏悬停提示功能为 spring-rs 的各种宏提供了详细的悬停信息，帮助开发者理解宏的作用和展开后的代码。

## 功能特性

### 1. Service 宏悬停提示

当用户悬停在 `#[derive(Service)]` 宏上时，显示：

- 宏的说明和用途
- 结构体名称
- 所有注入字段的详细信息
  - 字段名称和类型
  - 注入类型（组件或配置）
  - 组件名称（如果指定）
- 完整的展开后代码，包括：
  - 原始结构体定义
  - 生成的 `build()` 方法
  - 每个字段的注入逻辑

**示例**：

```rust
#[derive(Clone, Service)]
struct UserService {
    #[inject(component)]
    db: ConnectPool,
    
    #[inject(config)]
    config: UserConfig,
}
```

悬停时显示：
- 结构体信息
- 字段注入详情
- 生成的 `impl UserService { pub fn build(...) }` 代码

### 2. Inject 属性悬停提示

当用户悬停在 `#[inject]` 属性上时，显示：

- 属性的说明和用途
- 注入类型（组件或配置）
- 组件名称（如果指定）
- 注入代码示例
- 使用示例

**组件注入示例**：

```rust
#[inject(component = "primary")]
db: ConnectPool,
```

悬停时显示：
- 注入类型：组件 (Component)
- 组件名称：`"primary"`
- 注入代码：`app.get_component::<T>("primary")`
- 适用场景说明（多实例场景）

**配置注入示例**：

```rust
#[inject(config)]
config: AppConfig,
```

悬停时显示：
- 注入类型：配置 (Config)
- 注入代码：`app.get_config::<T>()`
- 配置文件说明（`config/app.toml`）

### 3. 路由宏悬停提示

当用户悬停在路由宏（`#[get]`、`#[post]` 等）上时，显示：

- 路由路径
- HTTP 方法列表
- 中间件列表（如果有）
- 处理器函数名称
- 展开后的路由注册代码

### 4. AutoConfig 宏悬停提示

当用户悬停在 `#[auto_config]` 宏上时，显示：

- 配置器类型
- 自动注册说明
- 展开后的配置代码

### 5. 任务调度宏悬停提示

支持三种任务调度宏：

**Cron 任务**：
- Cron 表达式
- 表达式格式说明
- 展开后的任务注册代码

**FixDelay 任务**：
- 延迟秒数
- 执行模式说明
- 展开后的任务注册代码

**FixRate 任务**：
- 频率秒数
- 执行模式说明
- 展开后的任务注册代码

## 实现细节

### API 设计

```rust
impl MacroAnalyzer {
    /// 为宏提供悬停提示
    pub fn hover_macro(&self, macro_info: &SpringMacro) -> String;
}
```

### 输出格式

悬停提示使用 Markdown 格式，包含：

- 标题（`#`）
- 粗体文本（`**text**`）
- 代码内联（`` `code` ``）
- 代码块（` ```rust ... ``` `）
- 列表（`-`）

### 代码复用

悬停提示功能复用了已实现的 `expand_macro()` 方法来生成展开后的代码，确保：

- 代码一致性
- 减少重复代码
- 易于维护

## 测试覆盖

实现了 18 个新的单元测试，覆盖：

- 所有宏类型的悬停提示
- 不同参数组合的悬停提示
- Markdown 格式验证
- 可读性验证
- 综合场景测试

所有测试都通过，总测试数量：74 个

## 使用示例

参见 `examples/hover_demo.rs`，展示了：

1. 各种宏的悬停提示生成
2. 完整的工作流（解析 -> 提取 -> 悬停）
3. 实际输出效果

运行示例：

```bash
cargo run --example hover_demo
```

## 与 LSP 集成

在 LSP 服务器中使用悬停提示功能：

```rust
// 当收到 hover 请求时
fn handle_hover(&self, params: HoverParams) -> Option<Hover> {
    // 1. 获取文档
    let doc = self.document_manager.get(&params.text_document.uri)?;
    
    // 2. 解析并提取宏
    let rust_doc = self.macro_analyzer.parse(doc.uri, doc.content)?;
    let result = self.macro_analyzer.extract_macros(rust_doc)?;
    
    // 3. 找到光标位置的宏
    let macro_at_position = find_macro_at_position(&result.macros, params.position)?;
    
    // 4. 生成悬停提示
    let hover_content = self.macro_analyzer.hover_macro(macro_at_position);
    
    // 5. 返回 LSP Hover 响应
    Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: hover_content,
        }),
        range: Some(macro_at_position.range()),
    })
}
```

## 未来改进

可能的改进方向：

1. **更丰富的文档**：
   - 添加更多使用场景说明
   - 添加常见错误和解决方案
   - 添加相关文档链接

2. **交互式示例**：
   - 提供可点击的代码片段
   - 支持快速插入代码

3. **性能优化**：
   - 缓存悬停提示结果
   - 延迟生成展开代码

4. **国际化**：
   - 支持多语言悬停提示
   - 根据用户设置选择语言

## 相关文档

- [宏分析器文档](../src/macro_analyzer.rs)
- [宏展开功能](./macro_expansion.md)
- [LSP 集成指南](./lsp_integration.md)

## 验证需求

此功能满足以下需求：

- **Requirements 7.2**: Service 宏悬停提示
- **Requirements 7.3**: Inject 属性悬停提示
- **Requirements 15.3**: 注入组件的悬停文档

对应的正确性属性：

- **Property 32**: Service 宏悬停提示
- **Property 33**: Inject 属性悬停提示
