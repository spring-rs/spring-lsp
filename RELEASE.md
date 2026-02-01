# spring-lsp 发布指南

本文档描述了 spring-lsp 项目的发布流程和相关工具。

## 🚀 快速发布

### 使用发布脚本（推荐）

```bash
# 发布补丁版本 (0.1.0 -> 0.1.1)
./scripts/release.sh patch

# 发布次要版本 (0.1.0 -> 0.2.0)
./scripts/release.sh minor

# 发布主要版本 (0.1.0 -> 1.0.0)
./scripts/release.sh major

# 发布预发布版本 (0.1.0 -> 0.2.0-alpha.1)
./scripts/release.sh prerelease alpha
```

### 使用 GitHub Actions

1. **自动版本更新**：
   - 访问 [Actions 页面](https://github.com/spring-rs/spring-lsp/actions/workflows/version-bump.yml)
   - 点击 "Run workflow"
   - 选择版本类型（patch/minor/major/prerelease）
   - 运行工作流

2. **手动发布**：
   - 访问 [Actions 页面](https://github.com/spring-rs/spring-lsp/actions/workflows/release.yml)
   - 点击 "Run workflow"
   - 输入版本号（如 0.1.0）
   - 选择是否为 dry run（测试模式）
   - 运行工作流

## 📋 发布流程

### 完整发布流程

1. **准备阶段**
   - 确保所有测试通过
   - 更新文档和示例
   - 检查代码质量（格式、Clippy）

2. **版本更新**
   - 更新 `Cargo.toml` 中的版本号
   - 更新 `CHANGELOG.md`
   - 创建发布提交和标签

3. **自动化发布**
   - GitHub Actions 自动触发
   - 构建多平台二进制文件
   - 发布到 crates.io
   - 创建 GitHub Release

4. **发布后验证**
   - 验证 crates.io 上的包
   - 测试安装和基本功能
   - 更新文档网站

### 版本号规则

spring-lsp 遵循 [语义化版本](https://semver.org/lang/zh-CN/) 规范：

- **主版本号**：不兼容的 API 修改
- **次版本号**：向下兼容的功能性新增
- **修订号**：向下兼容的问题修正

#### 版本示例

```
0.1.0       # 初始版本
0.1.1       # 补丁版本（Bug 修复）
0.2.0       # 次要版本（新功能）
1.0.0       # 主要版本（重大更改）
0.2.0-alpha.1   # 预发布版本
0.2.0-beta.1    # Beta 版本
0.2.0-rc.1      # Release Candidate
```

## 🛠️ 发布工具

### 1. 发布脚本 (`scripts/release.sh`)

本地发布脚本，提供完整的发布流程：

**功能：**
- 依赖检查（cargo、git、cargo-edit）
- Git 状态验证
- 代码质量检查（格式、Clippy、测试）
- 版本号更新
- CHANGELOG.md 更新
- 创建发布提交和标签
- 推送到远程仓库

**使用方法：**
```bash
# 基本用法
./scripts/release.sh <版本类型> [预发布类型]

# 示例
./scripts/release.sh patch           # 补丁版本
./scripts/release.sh minor           # 次要版本
./scripts/release.sh major           # 主要版本
./scripts/release.sh prerelease alpha # Alpha 预发布版本
./scripts/release.sh prerelease beta  # Beta 预发布版本
./scripts/release.sh prerelease rc    # RC 预发布版本
```

### 2. GitHub Actions Workflows

#### 发布工作流 (`.github/workflows/release.yml`)

**触发条件：**
- 推送标签（`v*.*.*`）
- 手动触发（workflow_dispatch）

**功能：**
- 预检查（代码质量、测试）
- 版本验证
- 多平台构建（Linux、macOS、Windows）
- 发布到 crates.io
- 创建 GitHub Release
- 上传二进制文件

**支持的平台：**
- Linux x86_64 (glibc)
- Linux x86_64 (musl)
- macOS x86_64
- macOS ARM64
- Windows x86_64

#### 版本更新工作流 (`.github/workflows/version-bump.yml`)

**触发条件：**
- 手动触发（workflow_dispatch）

**功能：**
- 自动更新版本号
- 更新 CHANGELOG.md
- 创建提交和标签
- 推送到远程仓库
- 创建 Pull Request（非补丁版本）

### 3. CI 工作流 (`.github/workflows/ci.yml`)

持续集成工作流，确保代码质量：

**功能：**
- 多平台测试（Ubuntu、macOS、Windows）
- 多 Rust 版本测试（stable、beta）
- 代码格式检查
- Clippy 静态分析
- 代码覆盖率报告

## 🔧 配置要求

### GitHub Secrets

发布流程需要以下 GitHub Secrets：

1. **`CARGO_REGISTRY_TOKEN`**
   - 用于发布到 crates.io
   - 在 [crates.io](https://crates.io/me) 生成 API Token
   - 添加到 GitHub 仓库的 Secrets

2. **`GITHUB_TOKEN`**
   - 用于创建 GitHub Release
   - GitHub 自动提供，无需手动配置

### 本地环境

发布脚本需要以下工具：

```bash
# 安装 Rust 和 Cargo
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装 cargo-edit（用于版本管理）
cargo install cargo-edit

# 确保 Git 已配置
git config --global user.name "Your Name"
git config --global user.email "your.email@example.com"
```

## 📝 发布检查清单

### 发布前检查

- [ ] 所有测试通过
- [ ] 代码格式正确 (`cargo fmt`)
- [ ] Clippy 检查通过 (`cargo clippy`)
- [ ] 文档构建成功 (`cargo doc`)
- [ ] 示例代码可运行
- [ ] README.md 已更新
- [ ] CHANGELOG.md 已准备

### 发布后验证

- [ ] crates.io 上的包可正常安装
- [ ] GitHub Release 已创建
- [ ] 二进制文件可下载并运行
- [ ] 文档网站已更新
- [ ] 版本标签已推送

## 🐛 故障排除

### 常见问题

1. **发布到 crates.io 失败**
   ```
   error: failed to publish to registry at https://crates.io/
   ```
   - 检查 `CARGO_REGISTRY_TOKEN` 是否正确配置
   - 确保版本号未被使用
   - 检查包名是否可用

2. **版本号冲突**
   ```
   error: crate version `0.1.0` is already uploaded
   ```
   - 更新版本号到未使用的版本
   - 检查 crates.io 上的现有版本

3. **构建失败**
   ```
   error: could not compile `spring-lsp`
   ```
   - 检查所有平台的兼容性
   - 确保依赖项可用
   - 检查 Rust 版本要求

4. **Git 推送失败**
   ```
   error: failed to push some refs
   ```
   - 确保有推送权限
   - 检查分支保护规则
   - 拉取最新更改

### 调试技巧

1. **使用 dry run 模式**
   ```bash
   # GitHub Actions 中启用 dry run
   # 不会实际发布，只测试流程
   ```

2. **本地测试构建**
   ```bash
   # 测试包构建
   cargo package --allow-dirty
   
   # 测试发布（不实际发布）
   cargo publish --dry-run
   ```

3. **检查生成的包**
   ```bash
   # 查看包内容
   cargo package --list
   
   # 解压查看
   tar -tzf target/package/spring-lsp-*.crate
   ```

## 📚 相关资源

- [Cargo 发布指南](https://doc.rust-lang.org/cargo/reference/publishing.html)
- [语义化版本规范](https://semver.org/lang/zh-CN/)
- [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)
- [GitHub Actions 文档](https://docs.github.com/en/actions)
- [crates.io 发布指南](https://doc.rust-lang.org/cargo/reference/publishing.html)

## 🤝 贡献

如果您发现发布流程中的问题或有改进建议，请：

1. 创建 Issue 描述问题
2. 提交 Pull Request 修复问题
3. 更新相关文档

---

**注意**：发布是一个重要操作，请在发布前仔细检查所有更改。建议先在测试环境中验证发布流程。