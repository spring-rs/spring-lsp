# spring-lsp

A Language Server Protocol (LSP) implementation for the [spring-rs](https://github.com/spring-rs/spring-rs) framework, providing intelligent IDE support for Rust applications built with spring-rs.

## Features

### 🎯 TOML Configuration Support
- **Smart completion** for configuration sections and properties
- **Real-time validation** with detailed error messages
- **Hover documentation** with type information and examples
- **Environment variable** support (`${VAR:default}` syntax)
- **Schema-based validation** with automatic schema loading

### 🔧 Rust Macro Analysis
- **Macro recognition** for spring-rs macros (`#[derive(Service)]`, `#[inject]`, route macros, job macros)
- **Macro expansion** with readable generated code
- **Parameter validation** and error reporting
- **Hover tooltips** with macro documentation and usage examples
- **Smart completion** for macro parameters

### 🌐 Route Management
- **Route detection** for all HTTP method macros (`#[get]`, `#[post]`, etc.)
- **Path parameter parsing** and validation
- **Conflict detection** for duplicate routes
- **Route navigation** and search capabilities
- **RESTful style validation**

### 🔍 Advanced Features
- **Dependency injection validation** with circular dependency detection
- **Component registration verification**
- **Performance monitoring** and server status queries
- **Configurable diagnostics** with custom filtering
- **Error recovery** with graceful degradation
- **Multi-document workspace** support

## Installation

### Prerequisites
- Rust 1.70+ 
- A compatible editor with LSP support (VS Code, Neovim, Emacs, etc.)

### From Source
```bash
git clone https://github.com/spring-rs/spring-lsp
cd spring-lsp
cargo build --release
```

The binary will be available at `target/release/spring-lsp`.

### From crates.io
```bash
cargo install spring-lsp
```

### Pre-built Binaries
Download pre-built binaries from the [releases page](https://github.com/spring-rs/spring-lsp/releases):

- Linux x86_64 (glibc and musl)
- macOS x86_64 and ARM64
- Windows x86_64

## Editor Setup

### VS Code
Install the spring-rs extension from the marketplace, or configure manually:

```json
{
  "spring-lsp.serverPath": "/path/to/spring-lsp",
  "spring-lsp.trace.server": "verbose"
}
```

### Neovim (with nvim-lspconfig)
```lua
require'lspconfig'.spring_lsp.setup{
  cmd = {"/path/to/spring-lsp"},
  filetypes = {"toml", "rust"},
  root_dir = require'lspconfig'.util.root_pattern("Cargo.toml", ".spring-lsp.toml"),
}
```

### Emacs (with lsp-mode)
```elisp
(add-to-list 'lsp-language-id-configuration '(toml-mode . "toml"))
(lsp-register-client
 (make-lsp-client :new-connection (lsp-stdio-connection "/path/to/spring-lsp")
                  :major-modes '(toml-mode rust-mode)
                  :server-id 'spring-lsp))
```

## Configuration

Create a `.spring-lsp.toml` file in your project root:

```toml
[completion]
trigger_characters = ["[", ".", "$", "{", "#", "("]

[schema]
url = "https://spring-rs.github.io/config-schema.json"

[diagnostics]
disabled = ["deprecated-config"]

[logging]
level = "info"
verbose = false
```

## Usage

### TOML Configuration Files
spring-lsp automatically provides intelligent support for `config/app.toml` and related configuration files:

```toml
# Smart completion for configuration sections
[web]
host = "0.0.0.0"  # Hover for documentation
port = 8080       # Type validation

# Environment variable support
[database]
url = "${DATABASE_URL:postgresql://localhost/mydb}"

# Validation and error reporting
[redis]
url = "redis://localhost:6379"
pool_size = 10    # Range validation
```

### Rust Code Analysis
spring-lsp analyzes your Rust code for spring-rs specific patterns:

```rust
// Service macro with dependency injection
#[derive(Clone, Service)]
struct UserService {
    #[inject(component)]
    db: ConnectPool,
    
    #[inject(config)]
    config: UserConfig,
}

// Route macros with validation
#[get("/users/{id}")]
async fn get_user(
    Path(id): Path<i64>,
    Component(service): Component<UserService>
) -> Result<Json<User>> {
    // Implementation
}

// Job scheduling macros
#[cron("0 0 * * * *")]
async fn cleanup_job() {
    // Hourly cleanup task
}
```

## Performance

spring-lsp is designed for high performance:

- **Startup time**: < 2 seconds
- **Completion response**: < 100ms
- **Diagnostic updates**: < 200ms
- **Memory usage**: < 50MB for typical projects
- **Concurrent documents**: 100+ supported

## Supported Features

| Feature | TOML | Rust | Status |
|---------|------|------|--------|
| Syntax highlighting | ✅ | ✅ | Complete |
| Completion | ✅ | ✅ | Complete |
| Hover documentation | ✅ | ✅ | Complete |
| Diagnostics | ✅ | ✅ | Complete |
| Go to definition | ⚠️ | ⚠️ | Partial |
| Document symbols | ⚠️ | ⚠️ | Planned |
| Workspace symbols | ⚠️ | ⚠️ | Planned |
| Code actions | ❌ | ❌ | Planned |
| Formatting | ❌ | ❌ | Planned |

## Architecture

spring-lsp follows a modular architecture:

```
┌─────────────────────────────────────────────────────────┐
│                    LSP Protocol Layer                    │
└─────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────┐
│                   Server Core Layer                      │
│         (Message Dispatch, State Management)             │
└─────────────────────────────────────────────────────────┘
                            ↓
┌──────────────┬──────────────┬──────────────┬────────────┐
│   Config     │    Macro     │   Routing    │ Diagnostic │
│   Analysis   │   Analysis   │   Analysis   │   Engine   │
└──────────────┴──────────────┴──────────────┴────────────┘
                            ↓
┌─────────────────────────────────────────────────────────┐
│                   Foundation Layer                       │
│      (Schema, Document, Index, Completion)              │
└─────────────────────────────────────────────────────────┘
```

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Setup
```bash
git clone https://github.com/spring-rs/spring-lsp
cd spring-lsp
cargo test
cargo run
```

### Running Tests
```bash
# Unit tests
cargo test --lib

# Integration tests  
cargo test --tests

# Property-based tests
cargo test --release

# Performance tests
cargo test --release performance
```

### Release Process
See [RELEASE.md](RELEASE.md) for detailed release instructions and tools.

## Documentation

- [User Guide](docs/user-guide.md) - Complete usage documentation
- [Configuration Reference](docs/configuration.md) - All configuration options
- [API Documentation](https://docs.rs/spring-lsp) - Rust API docs
- [Architecture Guide](docs/architecture.md) - Technical architecture
- [Contributing Guide](CONTRIBUTING.md) - Development guidelines

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for release history.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Acknowledgments

- [spring-rs](https://github.com/spring-rs/spring-rs) - The amazing Rust application framework
- [taplo](https://github.com/tamasfe/taplo) - TOML parsing and analysis
- [lsp-server](https://github.com/rust-lang/rust-analyzer/tree/master/lib/lsp-server) - LSP protocol implementation
- [rust-analyzer](https://github.com/rust-lang/rust-analyzer) - Inspiration for LSP architecture

---

**spring-lsp** - Intelligent IDE support for spring-rs applications