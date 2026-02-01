# spring-lsp 用户指南

本指南将帮助您快速上手 spring-lsp 语言服务器，为您的 spring-rs 项目提供强大的 IDE 支持。

## 快速开始

### 安装

#### 从源码构建
```bash
git clone https://github.com/spring-rs/spring-lsp
cd spring-lsp
cargo build --release
```

构建完成后，二进制文件位于 `target/release/spring-lsp`。

#### 从 crates.io 安装（即将支持）
```bash
cargo install spring-lsp
```

### 编辑器配置

#### VS Code
1. 安装 spring-rs 扩展（推荐）
2. 或手动配置：
```json
{
  "spring-lsp.serverPath": "/path/to/spring-lsp",
  "spring-lsp.trace.server": "verbose"
}
```

#### Neovim
```lua
require'lspconfig'.spring_lsp.setup{
  cmd = {"/path/to/spring-lsp"},
  filetypes = {"toml", "rust"},
  root_dir = require'lspconfig'.util.root_pattern("Cargo.toml", ".spring-lsp.toml"),
}
```

#### Emacs
```elisp
(lsp-register-client
 (make-lsp-client :new-connection (lsp-stdio-connection "/path/to/spring-lsp")
                  :major-modes '(toml-mode rust-mode)
                  :server-id 'spring-lsp))
```

## 功能详解

### TOML 配置支持

spring-lsp 为 spring-rs 配置文件提供全面支持：

#### 智能补全
- **配置节补全**：输入 `[` 后自动提示可用的配置节
- **配置项补全**：在配置节内提示可用的配置项
- **枚举值补全**：为枚举类型提供所有可能的值
- **环境变量补全**：支持 `${VAR:default}` 语法

```toml
# 输入 [ 后会提示：web, redis, database, logger 等
[web]
# 输入配置项时会提示：host, port, workers 等
host = "0.0.0.0"
port = 8080

# 环境变量支持
[database]
url = "${DATABASE_URL:postgresql://localhost/mydb}"
```

#### 实时验证
- **类型检查**：验证配置值的类型是否正确
- **必需项检查**：提示缺失的必需配置项
- **范围验证**：检查数值是否在有效范围内
- **废弃警告**：提示已废弃的配置项

#### 悬停文档
将鼠标悬停在配置项上可查看：
- 配置项的类型和描述
- 默认值和可选值
- 使用示例
- 相关文档链接

### Rust 宏分析

#### 支持的宏
- `#[derive(Service)]` - 服务组件定义
- `#[inject]` - 依赖注入
- `#[get]`, `#[post]`, `#[put]`, `#[delete]` - HTTP 路由
- `#[cron]`, `#[fix_delay]`, `#[fix_rate]` - 定时任务
- `#[auto_config]` - 自动配置

#### 宏展开
查看宏展开后的代码：
```rust
#[derive(Clone, Service)]
struct UserService {
    #[inject(component)]
    db: ConnectPool,
}

// 悬停在 Service 宏上可看到展开后的代码
```

#### 参数验证
- 验证宏参数的正确性
- 提供错误修复建议
- 检查路径参数语法

### 路由管理

#### 路由识别
自动识别所有路由定义：
```rust
#[get("/users/{id}")]
async fn get_user(Path(id): Path<i64>) -> Json<User> {
    // 实现
}
```

#### 路由验证
- **路径语法检查**：验证路径参数格式
- **冲突检测**：发现重复的路由定义
- **RESTful 风格**：建议符合 REST 规范的路径

#### 路由导航
- 快速查找所有路由
- 跳转到路由处理器
- 按路径或方法搜索

### 依赖注入验证

#### 组件验证
- 验证注入的组件是否已注册
- 检查组件类型是否匹配
- 验证组件名称是否正确

#### 循环依赖检测
自动检测并报告循环依赖：
```rust
// 会检测到 A -> B -> A 的循环依赖
#[derive(Service)]
struct ServiceA {
    #[inject(component)]
    b: ServiceB,
}

#[derive(Service)]
struct ServiceB {
    #[inject(component)]
    a: ServiceA,  // 循环依赖！
}
```

## 配置

### 配置文件

在项目根目录创建 `.spring-lsp.toml`：

```toml
[completion]
# 触发补全的字符
trigger_characters = ["[", ".", "$", "{", "#", "("]

[schema]
# Schema URL（用于配置验证）
url = "https://spring-rs.github.io/config-schema.json"

[diagnostics]
# 禁用的诊断类型
disabled = ["deprecated-config", "unused-import"]

[logging]
# 日志级别：error, warn, info, debug, trace
level = "info"
# 详细日志输出
verbose = false
```

### 环境变量

可以通过环境变量覆盖配置：

```bash
export SPRING_LSP_LOG_LEVEL=debug
export SPRING_LSP_SCHEMA_URL=https://my-custom-schema.json
```

## 性能优化

### 大型项目
对于大型项目，可以调整以下设置：

```toml
[performance]
# 文档缓存大小（MB）
cache_size = 100
# 并发分析线程数
worker_threads = 4
# 增量分析阈值（行数）
incremental_threshold = 1000
```

### 内存使用
- 典型项目：< 50MB
- 大型项目：< 200MB
- 支持 100+ 并发文档

## 故障排除

### 常见问题

#### 1. 服务器启动失败
```bash
# 检查二进制文件是否存在
ls -la /path/to/spring-lsp

# 检查权限
chmod +x /path/to/spring-lsp

# 查看详细错误
SPRING_LSP_LOG_LEVEL=debug /path/to/spring-lsp
```

#### 2. 补全不工作
- 确认文件类型为 `.toml` 或 `.rs`
- 检查项目根目录是否有 `Cargo.toml`
- 验证 Schema URL 是否可访问

#### 3. 诊断不准确
- 更新到最新版本的 Schema
- 检查配置文件语法是否正确
- 查看服务器日志获取详细信息

### 调试模式

启用详细日志：
```toml
[logging]
level = "debug"
verbose = true
```

或使用环境变量：
```bash
export SPRING_LSP_LOG_LEVEL=debug
export SPRING_LSP_VERBOSE=true
```

### 性能分析

查看服务器状态：
```bash
# 发送状态查询请求（需要编辑器支持）
# 或查看日志中的性能指标
```

## 高级用法

### 自定义 Schema

创建自定义配置 Schema：

```json
{
  "plugins": {
    "my-plugin": {
      "prefix": "my-plugin",
      "properties": {
        "enabled": {
          "type": "boolean",
          "description": "Enable my plugin",
          "default": true
        }
      }
    }
  }
}
```

### 插件开发

为 spring-lsp 开发插件：

```rust
// 实现自定义分析器
pub struct MyAnalyzer;

impl Analyzer for MyAnalyzer {
    fn analyze(&self, document: &Document) -> Vec<Diagnostic> {
        // 自定义分析逻辑
    }
}
```

### 集成测试

在 CI/CD 中使用 spring-lsp：

```yaml
- name: Validate configuration
  run: |
    spring-lsp --check config/app.toml
    spring-lsp --lint src/**/*.rs
```

## 最佳实践

### 项目结构
```
my-spring-project/
├── Cargo.toml
├── .spring-lsp.toml          # LSP 配置
├── config/
│   ├── app.toml              # 主配置
│   ├── app-dev.toml          # 开发环境
│   └── app-prod.toml         # 生产环境
└── src/
    ├── main.rs
    └── ...
```

### 配置组织
- 使用环境特定的配置文件
- 合理使用环境变量
- 为敏感信息使用外部配置

### 代码风格
- 保持宏使用的一致性
- 使用有意义的路由路径
- 合理组织依赖注入

## 更多资源

- [API 文档](https://docs.rs/spring-lsp)
- [spring-rs 框架文档](https://spring-rs.github.io/)
- [GitHub 仓库](https://github.com/spring-rs/spring-lsp)
- [问题反馈](https://github.com/spring-rs/spring-lsp/issues)

## 贡献

欢迎贡献代码！请查看 [CONTRIBUTING.md](../CONTRIBUTING.md) 了解详细信息。

---

如果您在使用过程中遇到问题，请随时在 GitHub 上创建 issue 或参与讨论。