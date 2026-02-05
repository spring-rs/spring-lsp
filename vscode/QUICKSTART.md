# 快速开始指南

## 开发扩展

### 1. 安装依赖

```bash
cd spring-lsp/vscode
npm install
```

### 2. 编译 TypeScript

```bash
npm run compile
```

或者启动监听模式：

```bash
npm run watch
```

### 3. 调试扩展

1. 在 VSCode 中打开 `spring-lsp/vscode` 目录
2. 按 `F5` 或点击 `Run > Start Debugging`
3. 这会打开一个新的 VSCode 窗口（Extension Development Host）
4. 在新窗口中打开一个包含 Spring RS 项目的文件夹
5. 打开 `.spring-lsp.toml` 或 `config/app.toml` 文件测试功能

### 4. 查看日志

在 Extension Development Host 窗口中：
1. 打开输出面板：`View > Output`
2. 从下拉菜单选择 `Spring RS Language Server`
3. 查看服务器通信日志

### 5. 打包扩展

```bash
npm run package
```

这会生成 `spring-rs-lsp-0.1.0.vsix` 文件。

### 6. 安装本地扩展

```bash
code --install-extension spring-rs-lsp-0.1.0.vsix
```

## 测试扩展

### 准备测试环境

1. 确保已安装 `spring-lsp` 服务器：
   ```bash
   cd spring-lsp
   cargo build --release
   # 将 target/release/spring-lsp 添加到 PATH
   ```

2. 创建一个测试项目：
   ```bash
   mkdir test-project
   cd test-project
   mkdir config
   ```

3. 创建 `config/app.toml`：
   ```toml
   #:schema https://spring-rs.github.io/config-schema.json
   
   [server]
   host = "127.0.0.1"
   port = 8080
   
   [logger]
   level = "debug"
   ```

4. 创建 `.spring-lsp.toml`：
   ```toml
   [server]
   enable = true
   ```

### 测试功能

1. **自动补全**
   - 在 `config/app.toml` 中输入 `[` 应该看到配置节的建议
   - 输入配置项名称应该看到补全选项

2. **悬停提示**
   - 将鼠标悬停在配置项上应该看到文档

3. **诊断**
   - 输入错误的配置值应该看到红色波浪线
   - 查看 Problems 面板应该看到错误信息

4. **跳转定义**
   - 右键点击配置项选择 "Go to Definition"

## 发布扩展

### 发布到 VSCode Marketplace

1. 创建 Azure DevOps 账号
2. 获取 Personal Access Token
3. 安装 vsce：
   ```bash
   npm install -g @vscode/vsce
   ```

4. 登录：
   ```bash
   vsce login spring-rs
   ```

5. 发布：
   ```bash
   vsce publish
   ```

### 发布到 Open VSX

```bash
npx ovsx publish spring-rs-lsp-0.1.0.vsix -p <token>
```

## 常见问题

### 服务器未启动

检查输出面板的错误信息：
- 确保 `spring-lsp` 在 PATH 中
- 或在设置中配置 `spring-rs-lsp.serverPath`

### 补全不工作

1. 检查文件是否匹配激活模式
2. 查看输出面板确认服务器已连接
3. 尝试重启 VSCode

### 修改代码后不生效

1. 确保运行了 `npm run compile`
2. 在 Extension Development Host 中重新加载窗口：`Ctrl+R` (Windows/Linux) 或 `Cmd+R` (macOS)

## 项目结构

```
vscode/
├── src/
│   └── extension.ts          # 扩展主入口
├── out/                       # 编译输出（自动生成）
├── node_modules/              # 依赖（自动生成）
├── .vscode/
│   ├── launch.json           # 调试配置
│   └── tasks.json            # 构建任务
├── package.json              # 扩展清单
├── tsconfig.json             # TypeScript 配置
├── .eslintrc.json            # ESLint 配置
├── .vscodeignore             # 打包时忽略的文件
├── README.md                 # 用户文档
├── CHANGELOG.md              # 更新日志
└── QUICKSTART.md             # 本文件
```

## 相关资源

- [VSCode 扩展 API](https://code.visualstudio.com/api)
- [Language Server Protocol](https://microsoft.github.io/language-server-protocol/)
- [vscode-languageclient](https://github.com/microsoft/vscode-languageserver-node)
