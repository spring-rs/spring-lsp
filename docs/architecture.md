# spring-lsp 架构文档

本文档描述了 spring-lsp 语言服务器的整体架构设计、核心组件和设计决策。

## 架构概览

spring-lsp 采用分层架构设计，从下到上分为四个主要层次：

```
┌─────────────────────────────────────────────────────────┐
│                    LSP Protocol Layer                    │
│              (lsp-server, JSON-RPC)                      │
└─────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────┐
│                   Server Core Layer                      │
│         (Message Dispatch, State Management)             │
│                    LspServer                             │
└─────────────────────────────────────────────────────────┘
                            ↓
┌──────────────┬──────────────┬──────────────┬────────────┐
│   Config     │    Macro     │   Routing    │ Diagnostic │
│   Analysis   │   Analysis   │   Analysis   │   Engine   │
│ TomlAnalyzer │MacroAnalyzer │RouteNavigator│DiagnosticE │
└──────────────┴──────────────┴──────────────┴────────────┘
                            ↓
┌─────────────────────────────────────────────────────────┐
│                   Foundation Layer                       │
│      (Schema, Document, Index, Completion)              │
└─────────────────────────────────────────────────────────┘
```

## 核心组件

### 1. LSP Protocol Layer

**职责**：处理 LSP 协议通信
- **lsp-server**：提供 LSP 协议的底层实现
- **JSON-RPC**：处理客户端-服务器通信
- **消息序列化**：JSON 消息的编码和解码

**关键特性**：
- 标准 LSP 3.17 协议支持
- 异步消息处理
- 错误恢复和重连机制

### 2. Server Core Layer

**职责**：服务器核心逻辑和状态管理

#### LspServer
```rust
pub struct LspServer {
    connection: Connection,
    state: ServerState,
    document_manager: Arc<DocumentManager>,
    // ... 其他组件
}
```

**核心功能**：
- **消息分发**：将 LSP 请求路由到相应的处理器
- **状态管理**：维护服务器生命周期状态
- **组件协调**：协调各个分析组件的工作
- **错误处理**：统一的错误处理和恢复策略

**状态机**：
```
Uninitialized → Initialized → ShuttingDown
```

### 3. Analysis Layer

#### TomlAnalyzer
**职责**：TOML 配置文件分析

```rust
pub struct TomlAnalyzer {
    schema_provider: SchemaProvider,
}
```

**功能**：
- TOML 语法解析（基于 taplo）
- 配置验证和错误检测
- 环境变量插值识别
- 悬停文档生成

#### MacroAnalyzer
**职责**：Rust 宏分析

```rust
pub struct MacroAnalyzer {
    // 内部状态
}
```

**功能**：
- Rust 语法解析（基于 syn）
- spring-rs 宏识别
- 宏展开和验证
- 参数补全支持

#### RouteNavigator
**职责**：路由管理和导航

```rust
pub struct RouteNavigator {
    route_index: Arc<RwLock<RouteIndex>>,
}
```

**功能**：
- 路由识别和索引
- 路径参数解析
- 冲突检测
- 路由搜索和导航

#### DiagnosticEngine
**职责**：诊断信息管理

```rust
pub struct DiagnosticEngine {
    diagnostics: DashMap<Url, Vec<Diagnostic>>,
}
```

**功能**：
- 诊断信息收集和存储
- 诊断发布和更新
- 诊断过滤和分类

### 4. Foundation Layer

#### DocumentManager
**职责**：文档生命周期管理

```rust
pub struct DocumentManager {
    documents: DashMap<Url, Document>,
}
```

**功能**：
- 文档缓存和版本管理
- 增量更新处理
- 并发访问控制

#### SchemaProvider
**职责**：配置 Schema 管理

```rust
pub struct SchemaProvider {
    schema_cache: DashMap<String, PluginSchema>,
    config_schema: Arc<RwLock<Option<ConfigSchema>>>,
}
```

**功能**：
- Schema 加载和缓存
- 网络 Schema 获取
- 降级策略实现

#### CompletionEngine
**职责**：智能补全功能

```rust
pub struct CompletionEngine {
    schema_provider: SchemaProvider,
}
```

**功能**：
- 上下文感知补全
- 补全项生成和排序
- 去重和过滤

#### IndexManager
**职责**：符号索引管理

```rust
pub struct IndexManager {
    symbol_index: Arc<RwLock<SymbolIndex>>,
    component_index: Arc<RwLock<ComponentIndex>>,
}
```

**功能**：
- 符号索引构建
- 增量索引更新
- 快速符号查找

## 设计原则

### 1. 模块化设计
- **单一职责**：每个组件专注于特定功能
- **松耦合**：组件间通过接口交互
- **高内聚**：相关功能集中在同一模块

### 2. 并发安全
- **无锁数据结构**：使用 DashMap 避免锁竞争
- **原子操作**：状态更新使用原子类型
- **读写分离**：使用 RwLock 优化读多写少场景

### 3. 性能优化
- **增量处理**：只重新分析修改的部分
- **智能缓存**：多层缓存策略
- **异步处理**：非阻塞 I/O 操作

### 4. 错误恢复
- **分层错误处理**：不同层次的错误处理策略
- **降级服务**：部分功能失败不影响整体
- **自动恢复**：网络错误等临时问题的自动重试

## 数据流

### 1. 文档打开流程
```
Client → LSP Server → DocumentManager → Analyzers → DiagnosticEngine → Client
```

1. 客户端发送 `textDocument/didOpen` 通知
2. 服务器更新文档缓存
3. 触发相关分析器分析文档
4. 收集诊断信息并发布给客户端

### 2. 补全请求流程
```
Client → LSP Server → CompletionEngine → SchemaProvider/Analyzers → Client
```

1. 客户端发送 `textDocument/completion` 请求
2. 服务器确定补全上下文
3. 调用相应的补全提供者
4. 返回补全项列表给客户端

### 3. 悬停请求流程
```
Client → LSP Server → Analyzers → SchemaProvider → Client
```

1. 客户端发送 `textDocument/hover` 请求
2. 服务器确定悬停位置的符号
3. 查询相关文档和信息
4. 返回格式化的悬停内容

## 配置系统

### 配置层次
1. **默认配置**：内置的默认值
2. **用户配置**：`~/.spring-lsp.toml`
3. **工作空间配置**：`.spring-lsp.toml`
4. **环境变量**：运行时覆盖

### 配置合并策略
```rust
final_config = default_config
    .merge(user_config)
    .merge(workspace_config)
    .merge(env_overrides)
```

## 扩展机制

### 1. 分析器扩展
```rust
pub trait Analyzer {
    fn analyze(&self, document: &Document) -> Vec<Diagnostic>;
    fn complete(&self, document: &Document, position: Position) -> Vec<CompletionItem>;
    fn hover(&self, document: &Document, position: Position) -> Option<Hover>;
}
```

### 2. Schema 扩展
- 支持自定义 Schema URL
- 插件特定的 Schema 定义
- Schema 合并和继承

### 3. 配置扩展
- 插件特定的配置节
- 动态配置更新
- 配置验证钩子

## 性能考虑

### 1. 内存管理
- **智能缓存**：LRU 缓存策略
- **内存池**：重用频繁分配的对象
- **弱引用**：避免循环引用导致的内存泄漏

### 2. CPU 优化
- **增量分析**：只分析变更部分
- **并行处理**：利用多核 CPU
- **懒加载**：按需加载资源

### 3. I/O 优化
- **异步 I/O**：非阻塞文件和网络操作
- **批量操作**：合并多个小操作
- **压缩传输**：减少网络传输量

## 测试架构

### 1. 测试层次
- **单元测试**：组件级别的功能测试
- **集成测试**：组件间交互测试
- **属性测试**：基于属性的随机测试
- **性能测试**：性能基准和回归测试

### 2. 测试工具
- **proptest**：属性测试框架
- **tokio-test**：异步代码测试
- **tempfile**：临时文件管理
- **pretty_assertions**：更好的断言输出

### 3. 测试策略
- **测试驱动开发**：先写测试再实现
- **持续集成**：自动化测试执行
- **覆盖率监控**：确保测试覆盖率

## 部署和运维

### 1. 构建系统
- **Cargo**：Rust 标准构建工具
- **交叉编译**：支持多平台构建
- **优化构建**：Release 模式优化

### 2. 监控和日志
- **结构化日志**：JSON 格式日志输出
- **性能指标**：内置性能监控
- **健康检查**：服务器状态查询

### 3. 配置管理
- **配置验证**：启动时配置检查
- **热重载**：运行时配置更新
- **环境适配**：不同环境的配置策略

## 未来扩展

### 1. 短期计划
- **定义跳转**：完善符号导航功能
- **代码操作**：快速修复和重构
- **格式化**：TOML 文件格式化

### 2. 长期规划
- **调试支持**：集成调试协议（DAP）
- **可视化工具**：依赖图和架构图
- **插件生态**：第三方插件支持

### 3. 性能优化
- **增量编译**：更快的重新分析
- **分布式缓存**：跨实例的缓存共享
- **机器学习**：智能补全和建议

## 总结

spring-lsp 的架构设计注重：

1. **可维护性**：清晰的模块边界和职责分离
2. **可扩展性**：插件化的分析器和配置系统
3. **高性能**：并发安全和增量处理
4. **可靠性**：全面的错误处理和测试覆盖

这种架构设计确保了 spring-lsp 能够为 spring-rs 开发者提供快速、准确、可靠的 IDE 支持。