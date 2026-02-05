# Spring RS Language Server for VSCode

为 Spring RS 框架提供智能的 TOML 配置文件支持。

## 功能特性

- ✅ **智能补全** - 为 Spring RS 配置文件提供自动补全
- ✅ **语法验证** - 实时检查配置文件的语法错误
- ✅ **悬停提示** - 显示配置项的文档和类型信息
- ✅ **跳转定义** - 快速跳转到配置项的定义
- ✅ **诊断信息** - 显示配置错误和警告

## 安装

### 前置要求

需要安装 `spring-lsp` 服务器：

```bash
# 使用 cargo 安装
cargo install spring-lsp

# 或从源码构建
git clone https://github.com/spring-rs/spring-lsp
cd spring-lsp
cargo build --release
```

### 安装扩展

1. 从 VSCode Marketplace 安装（即将推出）
2. 或从 VSIX 文件安装：
   ```bash
   code --install-extension spring-rs-lsp-0.1.0.vsix
   ```

## 使用方法

扩展会自动激活当检测到以下文件时：
- `.spring-lsp.toml` - Spring LSP 配置文件
- `config/app.toml` - Spring RS 应用配置文件
- `config/app-*.toml` - 环境特定的配置文件

## 配置选项

在 VSCode 设置中可以配置以下选项：

```json
{
  // 启用/禁用扩展
  "spring-rs-lsp.enable": true,
  
  // 自定义 spring-lsp 可执行文件路径
  "spring-rs-lsp.serverPath": "",
  
  // 调试：跟踪服务器通信
  "spring-rs-lsp.trace.server": "off"
}
```

## 支持的文件

- `.spring-lsp.toml` - LSP 配置
- `config/app.toml` - 主配置文件
- `config/app-dev.toml` - 开发环境配置
- `config/app-prod.toml` - 生产环境配置
- 其他 `config/app-*.toml` 文件

## 故障排除

### 服务器未启动

1. 检查 `spring-lsp` 是否已安装：
   ```bash
   spring-lsp --version
   ```

2. 如果未找到，请安装或配置路径：
   - 打开设置：`Ctrl+,` (Windows/Linux) 或 `Cmd+,` (macOS)
   - 搜索 `spring-rs-lsp.serverPath`
   - 设置 `spring-lsp` 可执行文件的完整路径

### 查看日志

1. 打开输出面板：`View` > `Output`
2. 选择 `Spring RS Language Server` 频道
3. 查看服务器日志和错误信息

### 启用调试模式

在设置中启用详细日志：

```json
{
  "spring-rs-lsp.trace.server": "verbose"
}
```

## 开发

### 构建扩展

```bash
cd vscode
npm install
npm run compile
```

### 打包扩展

```bash
npm run package
```

这将生成 `spring-rs-lsp-0.1.0.vsix` 文件。

### 本地测试

1. 在 VSCode 中打开 `vscode` 目录
2. 按 `F5` 启动调试
3. 在新窗口中打开一个 Spring RS 项目

## 相关链接

- [Spring RS 框架](https://github.com/spring-rs/spring-rs)
- [Spring LSP 服务器](https://github.com/spring-rs/spring-lsp)
- [问题反馈](https://github.com/spring-rs/spring-lsp/issues)

## 许可证

MIT OR Apache-2.0
