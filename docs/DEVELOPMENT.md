# 开发指南

## 代码格式化

本项目使用 `rustfmt` 保持代码风格一致。

### 手动格式化

```bash
# 格式化所有代码
cargo fmt --all

# 检查格式（不修改文件）
cargo fmt --all -- --check
```

### 自动格式化（Git Hooks）

为了确保提交前代码已格式化，可以安装 Git pre-commit hook：

```bash
# 运行安装脚本
./scripts/install-hooks.sh
```

这会在每次 `git commit` 前自动运行 `cargo fmt --all`。

### CI 检查

GitHub Actions 会在 PR 和 push 时自动检查代码格式。如果格式不正确，CI 会失败。

### 编辑器集成

#### VSCode

安装 `rust-analyzer` 扩展后，在 `.vscode/settings.json` 中添加：

```json
{
  "editor.formatOnSave": true,
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  }
}
```

#### Vim/Neovim

使用 `rust.vim` 插件：

```vim
let g:rustfmt_autosave = 1
```

#### IntelliJ IDEA / CLion

在 Settings → Languages & Frameworks → Rust → Rustfmt 中启用：
- ✓ Run rustfmt on Save

## 代码检查

```bash
# 运行 clippy
cargo clippy --all-features -- -D warnings

# 运行测试
cargo test --all-features
```

## 提交前检查清单

在提交代码前，请确保：

- [ ] 代码已格式化（`cargo fmt --all`）
- [ ] 通过 clippy 检查（`cargo clippy --all-features`）
- [ ] 所有测试通过（`cargo test --all-features`）
- [ ] 添加了必要的文档注释
- [ ] 更新了相关文档（如有需要）
