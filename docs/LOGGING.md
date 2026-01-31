# spring-lsp 日志系统

spring-lsp 使用 `tracing` 和 `tracing-subscriber` 实现结构化日志系统，支持灵活的配置和多种输出方式。

## 特性

- ✅ 通过环境变量配置日志级别
- ✅ 支持详细日志模式（verbose mode）
- ✅ 支持输出到标准错误流（stderr）
- ✅ 支持输出到文件（JSON 格式）
- ✅ 结构化日志（包含时间戳、级别、模块、线程等信息）
- ✅ 不干扰 LSP 协议通信（LSP 使用 stdin/stdout）
- ✅ 自动创建日志文件目录
- ✅ 终端颜色支持（自动检测）

## 环境变量配置

### SPRING_LSP_LOG_LEVEL

设置日志级别，支持以下值（不区分大小写）：

- `trace` - 最详细的日志，包含所有信息
- `debug` - 调试信息
- `info` - 一般信息（默认）
- `warn` - 警告信息
- `error` - 错误信息

**示例：**

```bash
# 设置为 debug 级别
export SPRING_LSP_LOG_LEVEL=debug

# 设置为 trace 级别（最详细）
export SPRING_LSP_LOG_LEVEL=trace

# 设置为 error 级别（只显示错误）
export SPRING_LSP_LOG_LEVEL=error
```

### SPRING_LSP_VERBOSE

启用详细日志模式，显示更多上下文信息：

- 目标模块名称
- 线程 ID 和线程名称
- 源文件名和行号

支持的值：`1`、`true`、`TRUE`（不区分大小写）

**示例：**

```bash
# 启用详细模式
export SPRING_LSP_VERBOSE=1

# 或者
export SPRING_LSP_VERBOSE=true
```

### SPRING_LSP_LOG_FILE

指定日志文件路径。如果设置，日志将同时输出到 stderr 和文件。

- stderr 输出：人类可读的格式，支持颜色（如果终端支持）
- 文件输出：JSON 格式，便于日志分析工具处理

**示例：**

```bash
# 输出到指定文件
export SPRING_LSP_LOG_FILE=/var/log/spring-lsp/server.log

# 输出到用户目录
export SPRING_LSP_LOG_FILE=~/.spring-lsp/logs/server.log

# 输出到临时目录
export SPRING_LSP_LOG_FILE=/tmp/spring-lsp.log
```

**注意：** 如果日志文件的父目录不存在，系统会自动创建。

## 使用示例

### 基本使用（默认配置）

```bash
# 使用默认配置启动（info 级别，输出到 stderr）
spring-lsp
```

### 调试模式

```bash
# 启用 debug 级别和详细模式
export SPRING_LSP_LOG_LEVEL=debug
export SPRING_LSP_VERBOSE=1
spring-lsp
```

### 输出到文件

```bash
# 输出到文件，便于后续分析
export SPRING_LSP_LOG_FILE=/var/log/spring-lsp/server.log
spring-lsp
```

### 完整配置示例

```bash
# 设置所有日志选项
export SPRING_LSP_LOG_LEVEL=debug
export SPRING_LSP_VERBOSE=1
export SPRING_LSP_LOG_FILE=~/.spring-lsp/logs/server.log

# 启动服务器
spring-lsp
```

### 在编辑器中配置

#### VSCode

在 VSCode 的 `settings.json` 中配置：

```json
{
  "spring-lsp.trace.server": "verbose",
  "spring-lsp.env": {
    "SPRING_LSP_LOG_LEVEL": "debug",
    "SPRING_LSP_VERBOSE": "1",
    "SPRING_LSP_LOG_FILE": "/tmp/spring-lsp.log"
  }
}
```

#### Neovim

在 Neovim 的 LSP 配置中：

```lua
require('lspconfig').spring_lsp.setup({
  cmd = { 'spring-lsp' },
  cmd_env = {
    SPRING_LSP_LOG_LEVEL = 'debug',
    SPRING_LSP_VERBOSE = '1',
    SPRING_LSP_LOG_FILE = '/tmp/spring-lsp.log',
  },
})
```

## 日志格式

### stderr 输出格式

人类可读的格式，包含时间戳、级别和消息：

```
2024-01-31T10:30:45.123456Z  INFO spring_lsp::server: Starting spring-lsp language server
2024-01-31T10:30:45.234567Z DEBUG spring_lsp::document: Document opened uri="file:///path/to/file.toml"
2024-01-31T10:30:45.345678Z  WARN spring_lsp::toml_analyzer: Unknown config key key="unknown.key"
2024-01-31T10:30:45.456789Z ERROR spring_lsp::server: Failed to parse document error="syntax error"
```

在详细模式下，还会显示模块、线程和源文件信息：

```
2024-01-31T10:30:45.123456Z  INFO spring_lsp::server [main] src/server.rs:123: Starting spring-lsp language server
```

### 文件输出格式（JSON）

结构化的 JSON 格式，便于日志分析：

```json
{
  "timestamp": "2024-01-31T10:30:45.123456Z",
  "level": "INFO",
  "target": "spring_lsp::server",
  "fields": {
    "message": "Starting spring-lsp language server"
  },
  "spans": []
}
```

## 日志级别说明

### TRACE

最详细的日志级别，包含所有操作的详细信息。适用于深度调试。

**输出内容：**
- 所有函数调用
- 所有数据结构的详细内容
- 所有中间计算结果

**使用场景：**
- 调试复杂的 bug
- 性能分析
- 理解代码执行流程

### DEBUG

调试信息，包含重要的中间状态和决策点。

**输出内容：**
- 重要的函数调用
- 关键的数据结构状态
- 决策点和分支选择

**使用场景：**
- 日常开发调试
- 功能验证
- 问题排查

### INFO（默认）

一般信息，记录重要的操作和状态变化。

**输出内容：**
- 服务器启动/停止
- 文档打开/关闭
- 重要的操作完成

**使用场景：**
- 生产环境
- 监控服务器状态
- 审计日志

### WARN

警告信息，表示潜在的问题或不推荐的操作。

**输出内容：**
- 配置错误（但可以继续运行）
- 废弃的 API 使用
- 性能问题警告

**使用场景：**
- 识别潜在问题
- 配置验证
- 代码质量检查

### ERROR

错误信息，表示操作失败或严重问题。

**输出内容：**
- 解析失败
- 网络错误
- 系统错误

**使用场景：**
- 错误监控
- 故障排查
- 问题报告

## 高级配置

### 使用 RUST_LOG 环境变量

如果设置了 `RUST_LOG` 环境变量，它会覆盖 `SPRING_LSP_LOG_LEVEL`。这允许更细粒度的控制：

```bash
# 只显示 spring_lsp 模块的 debug 日志
export RUST_LOG=spring_lsp=debug

# 显示 spring_lsp 的 debug 日志和 lsp_server 的 info 日志
export RUST_LOG=spring_lsp=debug,lsp_server=info

# 显示所有模块的 trace 日志
export RUST_LOG=trace
```

### 日志轮转

spring-lsp 本身不提供日志轮转功能，但你可以使用系统工具：

#### 使用 logrotate（Linux）

创建 `/etc/logrotate.d/spring-lsp`：

```
/var/log/spring-lsp/*.log {
    daily
    rotate 7
    compress
    delaycompress
    missingok
    notifempty
    create 0644 user group
}
```

#### 使用 newsyslog（macOS）

编辑 `/etc/newsyslog.conf`：

```
/var/log/spring-lsp/server.log  644  7  *  @T00  J
```

## 故障排查

### 日志文件未创建

**可能原因：**
1. 没有写入权限
2. 父目录不存在且无法创建
3. 磁盘空间不足

**解决方法：**
```bash
# 检查目录权限
ls -la /path/to/log/directory

# 手动创建目录
mkdir -p /path/to/log/directory

# 检查磁盘空间
df -h
```

### 日志级别不生效

**可能原因：**
1. 环境变量拼写错误
2. `RUST_LOG` 环境变量覆盖了配置
3. 日志系统已经被其他代码初始化

**解决方法：**
```bash
# 检查环境变量
env | grep SPRING_LSP
env | grep RUST_LOG

# 取消 RUST_LOG 设置
unset RUST_LOG
```

### 日志输出干扰 LSP 通信

**原因：** 日志输出到了 stdout 而不是 stderr。

**解决方法：** spring-lsp 已经确保所有日志输出到 stderr 或文件，不会干扰 LSP 协议。如果遇到问题，请检查是否有其他代码输出到 stdout。

## 性能考虑

### 日志级别对性能的影响

- `error`/`warn`: 几乎无影响
- `info`: 轻微影响（< 1%）
- `debug`: 中等影响（1-5%）
- `trace`: 显著影响（5-20%）

**建议：**
- 生产环境使用 `info` 或 `warn`
- 开发环境使用 `debug`
- 只在必要时使用 `trace`

### 文件输出的影响

文件输出是异步的，对性能影响很小（< 1%）。但要注意：

1. 磁盘空间：JSON 格式的日志文件可能很大
2. I/O 性能：使用 SSD 可以减少影响
3. 日志轮转：定期清理旧日志

## 最佳实践

1. **开发时使用 debug 级别**
   ```bash
   export SPRING_LSP_LOG_LEVEL=debug
   export SPRING_LSP_VERBOSE=1
   ```

2. **生产环境使用 info 级别**
   ```bash
   export SPRING_LSP_LOG_LEVEL=info
   ```

3. **问题排查时启用文件输出**
   ```bash
   export SPRING_LSP_LOG_FILE=/tmp/spring-lsp-debug.log
   export SPRING_LSP_LOG_LEVEL=debug
   ```

4. **使用日志分析工具**
   - `jq` 处理 JSON 日志
   - `grep` 过滤特定消息
   - `tail -f` 实时查看日志

5. **定期清理日志文件**
   ```bash
   # 删除 7 天前的日志
   find /var/log/spring-lsp -name "*.log" -mtime +7 -delete
   ```

## 相关资源

- [tracing 文档](https://docs.rs/tracing/)
- [tracing-subscriber 文档](https://docs.rs/tracing-subscriber/)
- [LSP 规范](https://microsoft.github.io/language-server-protocol/)
