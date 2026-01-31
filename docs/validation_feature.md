# 宏参数验证功能

## 概述

spring-lsp 的宏参数验证功能可以检查 spring-rs 宏的参数是否符合规范，帮助开发者在编译前发现配置错误。

## 支持的宏类型

### 1. Service 宏验证

验证 `#[derive(Service)]` 宏的结构体字段：

**验证规则：**
- 检查 `#[inject]` 属性的正确性
- 组件名称不能为空字符串
- Config 类型的注入不应该指定组件名称

**示例：**

```rust
// ✓ 正确
#[derive(Service)]
struct UserService {
    #[inject(component)]
    db: ConnectPool,
    
    #[inject(config)]
    config: UserConfig,
}

// ✗ 错误：组件名称为空字符串
#[derive(Service)]
struct UserService {
    #[inject(component = "")]  // 错误：E001
    db: ConnectPool,
}

// ✗ 错误：Config 注入不应该有组件名称
#[derive(Service)]
struct UserService {
    #[inject(config = "my_config")]  // 错误：E002
    config: UserConfig,
}
```

### 2. Inject 宏验证

验证 `#[inject]` 属性的参数：

**验证规则：**
- Config 类型的注入不应该指定组件名称
- 组件名称不能为空字符串

**错误代码：**
- `E002`: Config 注入不应该指定组件名称

### 3. AutoConfig 宏验证

验证 `#[auto_config]` 宏的配置器类型：

**验证规则：**
- 配置器类型不能为空

**示例：**

```rust
// ✓ 正确
#[auto_config(WebConfigurator)]
#[tokio::main]
async fn main() {
    App::new().run().await
}

// ✗ 错误：配置器类型为空
#[auto_config()]  // 错误：E003
#[tokio::main]
async fn main() {
    App::new().run().await
}
```

**错误代码：**
- `E003`: AutoConfig 宏必须指定配置器类型

### 4. Route 宏验证

验证路由宏（`#[get]`、`#[post]` 等）的参数：

**验证规则：**
- 路由路径不能为空
- 路由路径必须以 `/` 开头
- 路径参数必须符合 `{param}` 格式
- 路径参数名称只能包含字母、数字和下划线
- 路径参数不能嵌套
- 必须至少指定一个 HTTP 方法
- 处理器函数名称不能为空

**示例：**

```rust
// ✓ 正确
#[get("/users/{id}")]
async fn get_user(id: i64) -> Result<Json<User>> {
    Ok(Json(User::default()))
}

// ✗ 错误：路径为空
#[get("")]  // 错误：E004
async fn handler() {}

// ✗ 错误：路径不以 / 开头
#[get("users")]  // 错误：E005
async fn handler() {}

// ✗ 错误：路径参数名称包含非法字符
#[get("/users/{id-name}")]  // 错误：E011
async fn handler() {}

// ✗ 错误：路径参数嵌套
#[get("/users/{{id}}")]  // 错误：E008
async fn handler() {}

// ✗ 错误：路径参数未闭合
#[get("/users/{id")]  // 错误：E012
async fn handler() {}

// ✗ 错误：路径参数名称为空
#[get("/users/{}")]  // 错误：E010
async fn handler() {}
```

**错误代码：**
- `E004`: 路由路径不能为空
- `E005`: 路由路径必须以 '/' 开头
- `E006`: 路由必须至少指定一个 HTTP 方法
- `E007`: 路由处理器函数名称不能为空
- `E008`: 路径参数不能嵌套
- `E009`: 路径参数缺少开括号 '{'
- `E010`: 路径参数名称不能为空
- `E011`: 路径参数名称只能包含字母、数字和下划线
- `E012`: 路径参数缺少闭括号 '}'

### 5. Job 宏验证

验证任务调度宏（`#[cron]`、`#[fix_delay]`、`#[fix_rate]`）的参数：

**验证规则：**

#### Cron 任务
- Cron 表达式不能为空
- Cron 表达式必须包含 6 个部分（秒 分 时 日 月 星期）

#### FixDelay 任务
- 延迟秒数为 0 会产生警告（可能不是预期的行为）

#### FixRate 任务
- 频率秒数不能为 0

**示例：**

```rust
// ✓ 正确
#[cron("0 0 * * * *")]
async fn hourly_job() {
    println!("Running hourly");
}

// ✗ 错误：Cron 表达式为空
#[cron("")]  // 错误：E013
async fn job() {}

// ✗ 错误：Cron 表达式格式错误
#[cron("0 0 *")]  // 错误：E015（只有 3 个部分）
async fn job() {}

// ⚠ 警告：延迟秒数为 0
#[fix_delay(0)]  // 警告：W001
async fn job() {}

// ✗ 错误：频率秒数为 0
#[fix_rate(0)]  // 错误：E014
async fn job() {}
```

**错误代码：**
- `E013`: Cron 表达式不能为空
- `E014`: 频率秒数不能为 0
- `E015`: Cron 表达式应该包含 6 个部分
- `W001`: 延迟秒数为 0 可能不是预期的行为（警告）

## 使用方法

### 在代码中使用

```rust
use spring_lsp::macro_analyzer::*;

let analyzer = MacroAnalyzer::new();

// 解析 Rust 文件
let doc = analyzer.parse(uri, content)?;

// 提取宏
let result = analyzer.extract_macros(doc)?;

// 验证每个宏
for macro_item in &result.macros {
    let diagnostics = analyzer.validate_macro(macro_item);
    
    for diagnostic in diagnostics {
        println!("错误: {}", diagnostic.message);
        println!("位置: {:?}", diagnostic.range);
        println!("严重程度: {:?}", diagnostic.severity);
        println!("错误代码: {:?}", diagnostic.code);
    }
}
```

### 诊断信息结构

每个诊断信息包含以下字段：

- `range`: 错误在源代码中的位置范围
- `severity`: 严重程度（ERROR、WARNING、INFO、HINT）
- `code`: 错误代码（如 "E001"、"W001"）
- `source`: 来源（固定为 "spring-lsp"）
- `message`: 错误描述信息

## 错误代码参考

### 错误（Error）

| 代码 | 描述 |
|------|------|
| E001 | 组件名称不能为空字符串 |
| E002 | 配置注入 (config) 不应该指定组件名称 |
| E003 | AutoConfig 宏必须指定配置器类型 |
| E004 | 路由路径不能为空 |
| E005 | 路由路径必须以 '/' 开头 |
| E006 | 路由必须至少指定一个 HTTP 方法 |
| E007 | 路由处理器函数名称不能为空 |
| E008 | 路径参数不能嵌套 |
| E009 | 路径参数缺少开括号 '{' |
| E010 | 路径参数名称不能为空 |
| E011 | 路径参数名称只能包含字母、数字和下划线 |
| E012 | 路径参数缺少闭括号 '}' |
| E013 | Cron 表达式不能为空 |
| E014 | 频率秒数不能为 0 |
| E015 | Cron 表达式应该包含 6 个部分 |

### 警告（Warning）

| 代码 | 描述 |
|------|------|
| W001 | 延迟秒数为 0 可能不是预期的行为 |

## 最佳实践

1. **在保存文件时自动验证**：配置编辑器在保存 Rust 文件时自动运行验证
2. **集成到 CI/CD**：在持续集成流程中运行验证，确保代码质量
3. **及时修复错误**：根据错误代码和描述信息快速定位和修复问题
4. **关注警告**：警告虽然不会阻止编译，但可能指示潜在的问题

## 性能考虑

- 验证过程非常快速，通常在毫秒级完成
- 验证是增量的，只验证修改的宏
- 不会影响编辑器的响应速度

## 未来改进

计划在未来版本中添加：

1. **更详细的 Cron 表达式验证**：验证每个字段的值范围
2. **路径参数类型检查**：验证路径参数类型与函数参数类型是否匹配
3. **组件存在性检查**：验证注入的组件是否已注册
4. **配置项存在性检查**：验证注入的配置项是否在配置文件中定义
5. **循环依赖检测**：检测服务之间的循环依赖
6. **快速修复建议**：为常见错误提供自动修复建议

## 相关文档

- [宏展开功能](./macro_expansion_feature.md)
- [悬停提示功能](./hover_feature.md)
- [Requirements 7.4](../.kiro/specs/spring-lsp/requirements.md#requirement-7-宏展开和提示)
