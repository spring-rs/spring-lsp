# DocumentManager 实现总结

## 任务概述

任务 3.1：实现 DocumentManager 的完整功能，包括增量更新逻辑。

## 实现内容

### 1. 核心功能

DocumentManager 提供以下核心功能：

- **文档打开** (`open`): 缓存新打开的文档
- **文档修改** (`change`): 支持全量和增量更新
- **文档关闭** (`close`): 清理文档缓存
- **文档获取** (`get`): 返回文档的克隆副本
- **文档访问** (`with_document`): 提供只读访问以减少克隆开销

### 2. 增量更新实现

增量更新是本次实现的核心，包括以下关键功能：

#### 2.1 `apply_incremental_change` 方法

该方法负责将 LSP 的增量修改应用到文档内容：

```rust
fn apply_incremental_change(
    content: &mut String,
    range: lsp_types::Range,
    text: &str,
) -> Result<(), String>
```

**实现要点：**
- 验证范围的有效性
- 计算起始和结束位置的字节偏移
- 构建新内容（优化内存分配）
- 错误处理和降级策略

#### 2.2 `position_to_offset` 方法

将 LSP Position（行号和字符偏移）转换为字节偏移：

```rust
fn position_to_offset(content: &str, position: lsp_types::Position) -> Result<usize, String>
```

**实现要点：**
- 按行遍历内容
- 累计字节偏移（包括换行符）
- 处理文件末尾的特殊情况

#### 2.3 `char_offset_to_byte_offset` 方法

将字符偏移转换为字节偏移（处理 UTF-8 多字节字符）：

```rust
fn char_offset_to_byte_offset(line: &str, char_offset: usize) -> Result<usize, String>
```

**实现要点：**
- 正确处理 UTF-8 多字节字符
- 使用 `char.len_utf8()` 计算字节长度
- 允许在行尾的位置

### 3. 并发安全性

- 使用 `DashMap` 提供无锁并发访问
- `get()` 方法返回克隆以避免长时间持有锁
- `with_document()` 方法使用闭包模式减少锁持有时间

### 4. 错误处理

- 增量更新失败时自动降级到全量更新
- 详细的错误日志记录
- 范围验证和边界检查

## 测试覆盖

实现了 12 个单元测试，覆盖以下场景：

1. **基本操作测试**
   - `test_open_document`: 测试文档打开
   - `test_close_document`: 测试文档关闭
   - `test_with_document`: 测试只读访问

2. **全量更新测试**
   - `test_full_content_change`: 测试全量内容替换

3. **增量更新测试**
   - `test_incremental_change_single_line`: 单行修改
   - `test_incremental_change_multiline`: 多行修改
   - `test_incremental_change_insert`: 插入操作
   - `test_incremental_change_delete`: 删除操作
   - `test_incremental_change_utf8`: UTF-8 字符处理
   - `test_multiple_changes`: 多个连续修改

4. **辅助函数测试**
   - `test_position_to_offset`: 位置到偏移转换
   - `test_char_offset_to_byte_offset`: 字符到字节偏移转换

## 性能优化

1. **内存分配优化**
   - 在构建新内容时预分配容量
   - 使用 `String::with_capacity` 减少重新分配

2. **并发性能**
   - DashMap 提供接近无锁的性能
   - 克隆策略避免长时间持有锁

3. **降级策略**
   - 增量更新失败时自动降级到全量更新
   - 确保系统的鲁棒性

## 符合的需求

本实现满足以下需求：

- **Requirements 1.3**: 文档打开通知处理和缓存
- **Requirements 1.4**: 文档修改通知和增量更新
- **Requirements 1.5**: 文档关闭通知和缓存清理

## 验证结果

所有测试通过：
```
running 12 tests
test document::tests::test_char_offset_to_byte_offset ... ok
test document::tests::test_close_document ... ok
test document::tests::test_full_content_change ... ok
test document::tests::test_incremental_change_insert ... ok
test document::tests::test_incremental_change_delete ... ok
test document::tests::test_incremental_change_multiline ... ok
test document::tests::test_incremental_change_single_line ... ok
test document::tests::test_incremental_change_utf8 ... ok
test document::tests::test_open_document ... ok
test document::tests::test_multiple_changes ... ok
test document::tests::test_position_to_offset ... ok
test document::tests::test_with_document ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured
```

## 后续工作

DocumentManager 的基础实现已完成，后续可以考虑：

1. 添加文档内容的语法分析缓存
2. 实现文档变更事件通知机制
3. 添加文档版本冲突检测
4. 优化大文件的处理性能

## 参考

- LSP 规范: https://microsoft.github.io/language-server-protocol/
- 设计文档: `.kiro/specs/spring-lsp/design.md`
- 需求文档: `.kiro/specs/spring-lsp/requirements.md`
