# spring-lsp 配置指南

本文档介绍 spring-lsp 语言服务器的配置系统。

## 配置文件位置

spring-lsp 支持从以下位置读取配置文件（按优先级从低到高排序）：

1. **默认配置**：内置的默认配置
2. **用户配置**：`~/.config/spring-lsp/config.toml`
3. **工作空间配置**：`<workspace>/.spring-lsp.toml`
4. **环境变量**：环境变量会覆盖所有配置文件设置

## 配置文件格式

配置文件使用 TOML 格式，包含以下几个部分：

### 日志配置

```toml
[logging]
# 日志级别：trace, debug, info, warn, error
level = "info"

# 是否启用详细模式
verbose = false

# 日志文件路径（可选）
log_file = "/tmp/spring-lsp.log"
```

**环境变量覆盖**：
- `SPRING_LSP_LOG_LEVEL`: 日志级别
- `SPRING_LSP_VERBOSE`: 启用详细日志（设置为 `1` 或 `true`）
- `SPRING_LSP_LOG_FILE`: 日志文件路径

### 补全配置

```toml
[completion]
# 触发补全的字符列表
trigger_characters = ["[", ".", "$", "{", "#", "("]
```

**触发字符说明**：
- `[`: TOML 配置节
- `.`: 嵌套配置项
- `$`: 环境变量
- `{`: 环境变量插值
- `#`: 宏属性
- `(`: 宏参数

### 诊断配置

```toml
[diagnostics]
# 禁用特定类型的诊断
disabled = ["deprecated_warning", "restful_style"]
```

**可用的诊断类型**：
- `deprecated_warning`: 废弃配置项警告
- `restful_style`: RESTful 风格建议
- `type_mismatch`: 类型不匹配错误
- `missing_required`: 缺少必需配置项警告
- `route_conflict`: 路由冲突警告
- `circular_dependency`: 循环依赖警告

### Schema 配置

```toml
[schema]
# Schema URL（HTTP URL 或 file:// URL）
url = "https://spring-rs.github.io/config-schema.json"
```

**环境变量覆盖**：
- `SPRING_LSP_SCHEMA_URL`: Schema URL

**使用本地 Schema**：
```toml
[schema]
url = "file:///path/to/custom-schema.json"
```

## 配置示例

### 开发环境配置

适合开发时使用，启用详细日志和所有诊断：

```toml
[logging]
level = "debug"
verbose = true
log_file = "/tmp/spring-lsp-dev.log"

[completion]
trigger_characters = ["[", ".", "$", "{", "#", "("]

[diagnostics]
disabled = []

[schema]
url = "https://spring-rs.github.io/config-schema.json"
```

### 生产环境配置

适合生产环境，只记录重要信息，禁用风格检查：

```toml
[logging]
level = "warn"
verbose = false

[completion]
trigger_characters = ["[", ".", "$", "{", "#", "("]

[diagnostics]
disabled = ["restful_style", "deprecated_warning"]

[schema]
url = "https://spring-rs.github.io/config-schema.json"
```

### 离线环境配置

使用本地 Schema 文件，适合无网络环境：

```toml
[logging]
level = "info"
verbose = false

[completion]
trigger_characters = ["[", ".", "$", "{", "#", "("]

[diagnostics]
disabled = []

[schema]
url = "file:///opt/spring-lsp/schema.json"
```

## 配置验证

spring-lsp 会在启动时验证配置文件，如果配置无效会记录错误日志并使用默认配置。

**常见配置错误**：

1. **无效的日志级别**：
   ```
   Invalid log level: invalid. Valid levels are: trace, debug, info, warn, error
   ```

2. **空的触发字符列表**：
   ```
   Trigger characters list cannot be empty
   ```

3. **无效的 Schema URL**：
   ```
   Invalid Schema URL: ftp://example.com/schema.json. Must start with http://, https://, or file://
   ```

## 配置优先级

配置的优先级从低到高为：

1. 默认配置
2. 用户配置文件（`~/.config/spring-lsp/config.toml`）
3. 工作空间配置文件（`<workspace>/.spring-lsp.toml`）
4. 环境变量

后面的配置会覆盖前面的配置。

## 动态配置更新

目前 spring-lsp 只在启动时读取配置。如果修改了配置文件，需要重启语言服务器才能生效。

未来版本可能会支持动态配置更新。

## 故障排除

### 配置文件未生效

1. 检查配置文件路径是否正确
2. 检查配置文件格式是否正确（使用 TOML 验证工具）
3. 查看日志文件中的错误信息
4. 尝试使用环境变量覆盖配置

### 日志未输出

1. 检查日志级别是否设置正确
2. 检查日志文件路径是否有写入权限
3. 如果使用 stderr 输出，确保编辑器捕获了 stderr

### Schema 加载失败

1. 检查网络连接
2. 检查 Schema URL 是否正确
3. 尝试使用本地 Schema 文件
4. 查看日志中的详细错误信息

## 相关文档

- [日志系统文档](LOGGING.md)
- [Schema 提供者文档](schema_provider_implementation.md)
- [诊断引擎文档](diagnostic_engine_implementation.md)
