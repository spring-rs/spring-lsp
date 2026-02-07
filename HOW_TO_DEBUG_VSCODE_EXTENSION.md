# 如何调试 VSCode 扩展

## 🎯 快速开始（3 步）

```bash
# 1. 进入扩展目录并安装依赖
cd spring-lsp/vscode
npm install

# 2. 启动监听模式（自动编译）
npm run watch

# 3. 在 VSCode 中打开项目并按 F5
code .
# 然后按 F5 启动调试
```

就这么简单！新的 VSCode 窗口（Extension Development Host）会打开，你可以在其中测试扩展。

## 📖 详细文档

所有调试相关的文档都在 `vscode/` 目录下：

### 主要文档

1. **[DEBUG_EXTENSION.md](vscode/DEBUG_EXTENSION.md)** - 完整的调试指南
   - 前置要求
   - 3 种调试方法
   - 6 个常见调试场景
   - 10 个调试技巧
   - 常见问题解决

2. **[QUICK_REFERENCE.md](vscode/QUICK_REFERENCE.md)** - 快速参考卡片
   - 常用命令
   - 快捷键列表
   - 代码片段
   - 开发工作流

3. **[DEBUGGING_SETUP_COMPLETE.md](vscode/DEBUGGING_SETUP_COMPLETE.md)** - 配置总结
   - 完成的工作清单
   - 配置详解
   - 学习路径

## 🐛 调试配置

已配置 3 个调试选项（在 `.vscode/launch.json` 中）：

### 1. Run Extension
基本调试，在空白工作空间启动。

**使用场景**: 测试扩展激活、命令注册等基本功能。

### 2. Run Extension (with test project)
在指定的测试项目中启动。

**使用场景**: 测试应用检测、配置解析等真实场景。

**配置方法**: 编辑 `vscode/.vscode/launch.json`，修改测试项目路径：
```json
{
  "args": [
    "--extensionDevelopmentPath=${workspaceFolder}",
    "/path/to/your/spring-rs/project"  // 修改这里
  ]
}
```

### 3. Extension Tests
运行自动化测试。

**使用场景**: 调试测试代码。

## ⌨️ 常用快捷键

| 快捷键 | 功能 |
|--------|------|
| `F5` | 启动调试 |
| `Ctrl+Shift+F5` | 重启调试 |
| `Shift+F5` | 停止调试 |
| `F9` | 切换断点 |
| `F10` | 单步跳过 |
| `F11` | 单步进入 |

## 🔍 调试技巧

### 1. 设置断点
点击代码行号左侧设置断点（红点）。

### 2. 查看日志
- **Console**: 在扩展宿主窗口按 `Ctrl+Shift+I`
- **Output**: View → Output → 选择 "Spring LSP"

### 3. 使用 Debug Console
在 Debug Console 中可以执行表达式：
```typescript
app                    // 查看变量
app.reset()           // 调用方法
this.apps.size        // 查看属性
```

### 4. 热重载
修改代码后：
1. 保存文件（`Ctrl+S`）
2. 在扩展宿主窗口按 `Ctrl+R` 重新加载

## 🛠️ 常用命令

```bash
# 编译 TypeScript
npm run compile

# 监听模式（自动编译）
npm run watch

# 运行测试
npm run test

# 代码检查
npm run lint

# 清理构建产物
npm run clean

# 验证配置
npm run verify

# 打包扩展
npm run package
```

## 🐛 常见问题

### Q: 按 F5 没反应？
**A**: 检查 Problems 面板是否有编译错误，运行 `npm run compile` 查看详情。

### Q: 断点不生效（灰色）？
**A**: 确保 `tsconfig.json` 中 `sourceMap: true`，重新编译并重启调试。

### Q: 修改代码不生效？
**A**: 确保运行了 `npm run watch`，或手动 `npm run compile`，然后按 `Ctrl+Shift+F5` 重启。

### Q: 扩展未激活？
**A**: 确保工作空间包含 `Cargo.toml` 或 `.spring-lsp.toml` 文件。

### Q: 语言服务器未启动？
**A**: 先编译语言服务器：
```bash
cd spring-lsp
cargo build --release
```

## 📂 项目结构

```
spring-lsp/
├── vscode/                          # VSCode 扩展
│   ├── src/                         # TypeScript 源码
│   │   ├── extension.ts             # 扩展入口
│   │   ├── controllers/             # 应用管理
│   │   ├── views/                   # 树视图
│   │   ├── languageClient/          # LSP 客户端
│   │   └── ...
│   ├── .vscode/
│   │   ├── launch.json              # 调试配置 ⭐
│   │   └── tasks.json               # 任务配置
│   ├── scripts/
│   │   ├── clean.sh                 # 清理脚本
│   │   └── verify.js                # 验证脚本
│   ├── DEBUG_EXTENSION.md           # 完整调试指南 ⭐
│   ├── QUICK_REFERENCE.md           # 快速参考 ⭐
│   ├── package.json                 # 扩展配置
│   └── tsconfig.json                # TypeScript 配置
└── src/                             # Rust 语言服务器
    └── ...
```

## 🎓 学习路径

### 第 1 步: 快速开始
1. 运行 `npm install` 和 `npm run watch`
2. 按 F5 启动调试
3. 在新窗口中测试扩展

### 第 2 步: 设置断点
1. 在 `src/extension.ts` 的 `activate()` 函数设置断点
2. 重启调试，观察激活过程
3. 查看变量和调用栈

### 第 3 步: 深入学习
1. 阅读 [DEBUG_EXTENSION.md](vscode/DEBUG_EXTENSION.md)
2. 学习 6 个常见调试场景
3. 掌握高级调试技巧

### 第 4 步: 实战练习
1. 修改代码添加新功能
2. 使用断点调试问题
3. 编写测试并调试

## 📚 相关资源

- **完整调试指南**: [vscode/DEBUG_EXTENSION.md](vscode/DEBUG_EXTENSION.md)
- **快速参考**: [vscode/QUICK_REFERENCE.md](vscode/QUICK_REFERENCE.md)
- **配置总结**: [vscode/DEBUGGING_SETUP_COMPLETE.md](vscode/DEBUGGING_SETUP_COMPLETE.md)
- **VSCode API**: https://code.visualstudio.com/api
- **LSP 规范**: https://microsoft.github.io/language-server-protocol/

## 💡 提示

1. ✅ 始终使用 `npm run watch` 自动编译
2. ✅ 善用断点和 Debug Console
3. ✅ 查看 Output 面板的日志
4. ✅ 使用 `Ctrl+R` 快速重新加载
5. ✅ 运行 `npm run verify` 验证配置

## 🎉 开始调试

现在你已经了解了基本知识，可以开始调试了：

```bash
cd spring-lsp/vscode
npm install
npm run watch
# 在 VSCode 中打开项目
code .
# 按 F5 启动调试
```

祝调试顺利！如有问题，请查看 [DEBUG_EXTENSION.md](vscode/DEBUG_EXTENSION.md) 的详细说明。

---

**快速指南 - 3 步开始调试！** 🚀
