# spring-lsp

Language Server Protocol implementation for [spring-rs](https://github.com/spring-rs/spring-rs) framework.

## 功能特性

spring-lsp 为 spring-rs 框架提供智能的开发体验：

- **TOML 配置文件支持**
  - 智能补全配置项和值
  - 实时验证配置正确性
  - 环境变量插值支持
  - 悬停显示配置文档

- **Rust 宏分析**
  - 识别 spring-rs 特定宏
  - 宏展开和提示
  - 宏参数补全和验证

- **路由管理**
  - 识别和索引所有路由
  - 路由导航和查找
  - 路由冲突检测
  - RESTful 风格检查

- **依赖注入验证**
  - 验证组件注册
  - 检测循环依赖
  - 配置注入验证

## 安装

### 从源码构建

```bash
git clone https://github.com/spring-rs/spring-lsp.git
cd spring-lsp
cargo build --release
```

构建完成后，可执行文件位于 `target/release/spring-lsp`。

### 从 crates.io 安装

```bash
cargo install spring-lsp
```

## 编辑器集成

### Visual Studio Code

1. 安装 spring-rs 扩展（即将推出）
2. 扩展会自动下载和配置 spring-lsp

### Vim/Neovim

使用 [nvim-lspconfig](https://github.com/neovim/nvim-lspconfig)：

```lua
require'lspconfig'.spring_lsp.setup{
  cmd = { "spring-lsp" },
  filetypes = { "rust", "toml" },
  root_dir = require'lspconfig'.util.root_pattern("Cargo.toml"),
}
```

### Emacs

使用 [lsp-mode](https://github.com/emacs-lsp/lsp-mode)：

```elisp
(add-to-list 'lsp-language-id-configuration '(rust-mode . "rust"))
(add-to-list 'lsp-language-id-configuration '(toml-mode . "toml"))

(lsp-register-client
 (make-lsp-client :new-connection (lsp-stdio-connection "spring-lsp")
                  :major-modes '(rust-mode toml-mode)
                  :server-id 'spring-lsp))
```

## 配置

spring-lsp 支持以下配置选项（通过 LSP 初始化参数）：

```json
{
  "spring-lsp": {
    "schemaUrl": "https://spring-rs.github.io/config-schema.json",
    "logLevel": "info",
    "completionTriggerCharacters": ["[", ".", "${"],
    "disabledDiagnostics": []
  }
}
```

## 开发

### 构建

```bash
cargo build
```

### 测试

```bash
# 运行所有测试
cargo test

# 运行单元测试
cargo test --lib

# 运行属性测试
cargo test --test '*'
```

### 日志

设置 `RUST_LOG` 环境变量来控制日志级别：

```bash
RUST_LOG=debug spring-lsp
```

## 架构

spring-lsp 采用分层架构：

```
┌─────────────────────────────────────────────────────────┐
│                    LSP Protocol Layer                    │
│              (lsp-server, JSON-RPC)                      │
└─────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────┐
│                   Server Core Layer                      │
│         (Message Dispatch, State Management)             │
└─────────────────────────────────────────────────────────┘
                            ↓
┌──────────────┬──────────────┬──────────────┬────────────┐
│   Config     │    Macro     │   Routing    │ Diagnostic │
│   Analysis   │   Analysis   │   Analysis   │   Engine   │
└──────────────┴──────────────┴──────────────┴────────────┘
                            ↓
┌─────────────────────────────────────────────────────────┐
│                   Foundation Layer                       │
│      (TOML Parser, Rust Parser, Cache, Index)           │
└─────────────────────────────────────────────────────────┘
```

详细的设计文档请参考 [design.md](.kiro/specs/spring-lsp/design.md)。

## 贡献

欢迎贡献！请查看 [CONTRIBUTING.md](CONTRIBUTING.md) 了解如何参与项目。

## 许可证

本项目采用 MIT 或 Apache-2.0 双重许可。详见 [LICENSE-MIT](LICENSE-MIT) 和 [LICENSE-APACHE](LICENSE-APACHE)。

## 相关项目

- [spring-rs](https://github.com/spring-rs/spring-rs) - Rust 应用框架
- [rust-analyzer](https://github.com/rust-lang/rust-analyzer) - Rust 语言服务器
- [taplo](https://github.com/tamasfe/taplo) - TOML 工具包
