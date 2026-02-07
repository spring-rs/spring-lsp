# 诊断引擎模块

## 当前实现

- **engine_impl.rs** - 当前使用的完整实现，包含所有诊断逻辑

## 未来重构计划

将 `engine_impl.rs` 的功能拆分为：

- **engine.rs** - 核心诊断引擎
- **validators.rs** - 各种验证器
  - TomlSyntaxValidator - TOML 语法验证
  - SchemaValidator - Schema 验证
  - RouteConflictValidator - 路由冲突验证
  - DependencyValidator - 依赖验证

## 使用方式

```rust
use spring_lsp::analysis::DiagnosticEngine;

let engine = DiagnosticEngine::new();
engine.validate_document(&uri, content);
engine.publish(&connection, &uri);
```

## 重构步骤

1. 保持 `engine_impl.rs` 不变（确保功能正常）
2. 在 `engine.rs` 中创建新的简化接口
3. 在 `validators.rs` 中实现各种验证器
4. 逐步迁移功能
5. 测试通过后，删除 `engine_impl.rs`
