# 发布 Pull Request

## 发布信息

- **版本：** v0.x.x
- **类型：** [patch/minor/major/prerelease]
- **发布日期：** YYYY-MM-DD

## 更改摘要

### 新增功能
- [ ] 功能 1
- [ ] 功能 2

### 改进
- [ ] 改进 1
- [ ] 改进 2

### 修复
- [ ] 修复 1
- [ ] 修复 2

### 破坏性更改
- [ ] 无破坏性更改
- [ ] 更改 1（如有）

## 发布前检查清单

### 代码质量
- [ ] 所有测试通过 (`cargo test --all-features`)
- [ ] 代码格式正确 (`cargo fmt --all -- --check`)
- [ ] Clippy 检查通过 (`cargo clippy --all-features -- -D warnings`)
- [ ] 文档构建成功 (`cargo doc --all-features --no-deps`)

### 版本管理
- [ ] `Cargo.toml` 版本号已更新
- [ ] `CHANGELOG.md` 已更新
- [ ] 版本号符合语义化版本规范

### 文档
- [ ] README.md 已更新（如需要）
- [ ] API 文档已更新
- [ ] 示例代码可运行

### 测试
- [ ] 单元测试覆盖新功能
- [ ] 集成测试通过
- [ ] 性能测试通过（如适用）
- [ ] 手动测试关键功能

### 发布准备
- [ ] 发布说明已准备
- [ ] 迁移指南已准备（如有破坏性更改）
- [ ] 相关 Issue 已关联

## 发布后计划

- [ ] 验证 crates.io 发布
- [ ] 测试安装和基本功能
- [ ] 更新相关文档
- [ ] 通知相关用户/项目

## 相关链接

- 相关 Issue: #xxx
- 里程碑: [vX.X.X](link)
- 文档: [链接](link)

## 审查者注意事项

请重点检查：
1. 版本号是否正确
2. CHANGELOG.md 是否完整
3. 破坏性更改是否有充分说明
4. 测试覆盖是否充分

---

/cc @maintainer1 @maintainer2