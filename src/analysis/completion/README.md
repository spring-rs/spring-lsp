# 补全引擎模块

## 当前实现

- **engine_impl.rs** - 当前使用的完整实现，包含所有补全逻辑

## 未来重构计划

将 `engine_impl.rs` 的功能拆分为：

- **engine.rs** - 核心补全引擎
- **providers.rs** - 各种补全提供器
  - TomlCompletionProvider - TOML 配置补全
  - MacroCompletionProvider - Rust 宏补全
  - EnvVarCompletionProvider - 环境变量补全

## 使用方式

```rust
use spring_lsp::analysis::CompletionEngine;

let engine = CompletionEngine::new(schema_provider);
let items = engine.complete_toml_document(&toml_doc, position);
```

## 重构步骤

1. 保持 `engine_impl.rs` 不变（确保功能正常）
2. 在 `engine.rs` 中创建新的简化接口
3. 在 `providers.rs` 中实现各种提供器
4. 逐步迁移功能
5. 测试通过后，删除 `engine_impl.rs`
