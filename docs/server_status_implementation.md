# 服务器状态查询实现文档

## 概述

本文档描述了 spring-lsp 服务器状态查询功能的实现，该功能允许客户端查询服务器的运行状态和性能指标。

## 实现的功能

### 1. 状态跟踪模块 (`src/status.rs`)

创建了 `ServerStatus` 结构体，使用原子操作跟踪服务器状态：

```rust
pub struct ServerStatus {
    start_time: Arc<Instant>,
    document_count: Arc<AtomicUsize>,
    request_count: Arc<AtomicU64>,
    error_count: Arc<AtomicU64>,
    completion_count: Arc<AtomicU64>,
    hover_count: Arc<AtomicU64>,
    diagnostic_count: Arc<AtomicU64>,
}
```

**关键特性：**
- 使用原子操作确保线程安全
- 可克隆，支持在多个组件间共享
- 提供增量更新方法（increment/decrement/record）
- 计算派生指标（错误率、每秒请求数）

### 2. 性能指标 (`ServerMetrics`)

定义了 `ServerMetrics` 结构体，包含以下指标：

- `uptime_seconds`: 服务器运行时长（秒）
- `document_count`: 当前打开的文档数量
- `request_count`: 总请求数
- `error_count`: 总错误数
- `completion_count`: 补全请求数
- `hover_count`: 悬停请求数
- `diagnostic_count`: 诊断发布数
- `requests_per_second`: 每秒请求数
- `error_rate`: 错误率（错误数/总请求数）

**特性：**
- 可序列化为 JSON（使用 serde）
- 提供人类可读的格式化输出
- 自动计算派生指标

### 3. LSP 服务器集成

在 `LspServer` 中集成了状态跟踪：

```rust
pub struct LspServer {
    // ... 其他字段
    pub status: ServerStatus,
}
```

**集成点：**

1. **文档管理**：
   - 文档打开时：`status.increment_document_count()`
   - 文档关闭时：`status.decrement_document_count()`

2. **请求处理**：
   - 每个请求：`status.record_request()`
   - 错误发生时：`status.record_error()`

3. **状态查询请求**：
   - 方法：`spring-lsp/status`
   - 返回：`ServerMetrics` 的 JSON 表示

### 4. 状态查询处理器

实现了 `handle_status_query` 方法：

```rust
fn handle_status_query(&self, req: Request) -> Result<()> {
    let metrics = self.status.get_metrics();
    let result = serde_json::to_value(metrics)?;
    
    let response = Response {
        id: req.id,
        result: Some(result),
        error: None,
    };
    
    self.connection.sender.send(Message::Response(response))?;
    Ok(())
}
```

## 使用示例

### 客户端请求

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "spring-lsp/status"
}
```

### 服务器响应

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "uptime_seconds": 3600,
    "document_count": 5,
    "request_count": 1234,
    "error_count": 12,
    "completion_count": 456,
    "hover_count": 234,
    "diagnostic_count": 89,
    "requests_per_second": 0.34,
    "error_rate": 0.0097
  }
}
```

## 测试

### 单元测试

在 `src/status.rs` 中实现了 13 个单元测试：

- 状态创建和初始化
- 文档计数增减
- 各种操作的跟踪
- 运行时长计算
- 指标格式化
- 并发更新安全性
- 零除保护

### 属性测试

在 `tests/status_query_property_test.rs` 中实现了 5 个属性测试：

**Property 56: 服务器状态查询**
- 验证状态查询返回正确的指标
- 验证指标可序列化/反序列化
- 验证格式化输出的可读性
- 验证文档计数不会为负
- 验证错误率计算的正确性

**测试策略：**
- 使用 proptest 生成随机操作序列
- 验证指标与实际操作一致
- 确保错误率在 0-1 范围内
- 验证并发安全性

## 性能考虑

1. **原子操作**：使用 `AtomicUsize` 和 `AtomicU64` 确保无锁并发访问
2. **内存效率**：使用 `Arc` 共享数据，避免不必要的复制
3. **计算效率**：派生指标在查询时计算，避免持续更新开销

## 线程安全

- 所有计数器使用原子操作，支持并发更新
- `ServerStatus` 实现了 `Clone`，可以在多个线程间共享
- 使用 `Ordering::Relaxed` 以获得最佳性能（计数器不需要严格顺序）

## 未来改进

1. **更多指标**：
   - 平均响应时间
   - 内存使用情况
   - 索引大小
   - 缓存命中率

2. **历史数据**：
   - 保存历史指标用于趋势分析
   - 提供时间序列数据

3. **性能分析**：
   - 按请求类型分类统计
   - 慢请求追踪
   - 性能瓶颈识别

4. **健康检查**：
   - 定义健康阈值
   - 自动检测异常状态
   - 提供健康状态端点

## 验证需求

本实现验证了以下需求：

- **Requirement 13.5**: WHEN 用户请求诊断信息时，THE LSP_Server SHALL 提供服务器状态和性能指标
- **Property 56**: For any 服务器状态查询请求，应该返回包含服务器状态和性能指标的响应

## 相关文件

- `src/status.rs` - 状态跟踪模块
- `src/server.rs` - LSP 服务器集成
- `tests/status_query_property_test.rs` - 属性测试
- `.kiro/specs/spring-lsp/requirements.md` - 需求文档
- `.kiro/specs/spring-lsp/design.md` - 设计文档
- `.kiro/specs/spring-lsp/tasks.md` - 任务列表

## 总结

服务器状态查询功能已完整实现，包括：

✅ 状态跟踪模块（原子操作，线程安全）  
✅ 性能指标计算（错误率、RPS 等）  
✅ LSP 请求处理器  
✅ JSON 序列化支持  
✅ 人类可读的格式化输出  
✅ 完整的单元测试（13 个测试）  
✅ 属性测试（5 个属性，100+ 迭代）  
✅ 并发安全验证  

所有测试通过，功能已准备好集成到生产环境。
