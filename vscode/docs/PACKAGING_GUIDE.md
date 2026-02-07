# VSCode 扩展打包指南

## 概述

Spring LSP 扩展支持两种使用方式：

1. **捆绑模式**（推荐）- 语言服务器二进制文件打包在扩展中
2. **独立安装模式** - 用户需要单独安装 spring-lsp

## 方案对比

| 特性 | 捆绑模式 | 独立安装模式 |
|------|---------|-------------|
| 用户体验 | ⭐⭐⭐⭐⭐ 开箱即用 | ⭐⭐⭐ 需要额外步骤 |
| 安装复杂度 | 简单 | 中等 |
| 扩展大小 | 较大（~10-30MB） | 很小（~1MB） |
| 更新方式 | 随扩展更新 | 需要单独更新 |
| 版本一致性 | ✅ 保证一致 | ⚠️ 可能不一致 |
| 适用场景 | 普通用户 | 开发者 |

## 推荐方案：捆绑模式

### 优点
- ✅ 用户安装扩展后即可使用，无需额外配置
- ✅ 版本一致性有保证
- ✅ 更新方便（扩展和服务器一起更新）
- ✅ 适合大多数用户

### 缺点
- ❌ 扩展包体积较大
- ❌ 需要为多个平台构建

## 实现方案

### 1. 项目结构

```
spring-lsp/
├── vscode/
│   ├── bin/                    # 语言服务器二进制文件（打包时添加）
│   │   ├── spring-lsp-linux-x64
│   │   ├── spring-lsp-darwin-x64
│   │   ├── spring-lsp-darwin-arm64
│   │   └── spring-lsp-win32-x64.exe
│   ├── src/
│   ├── scripts/
│   │   ├── build-server.sh     # 构建语言服务器
│   │   ├── package-extension.sh # 打包扩展
│   │   └── download-server.sh  # 下载预构建的服务器
│   ├── package.json
│   └── .vscodeignore
└── Cargo.toml
```

### 2. 构建脚本

#### `scripts/build-server.sh`

```bash
#!/bin/bash

# 构建语言服务器的所有平台版本

set -e

echo "🔨 Building Spring LSP server for all platforms..."

# 进入语言服务器目录
cd ..

# 定义目标平台
TARGETS=(
  "x86_64-unknown-linux-gnu"
  "x86_64-apple-darwin"
  "aarch64-apple-darwin"
  "x86_64-pc-windows-msvc"
)

# 创建 bin 目录
mkdir -p vscode/bin

# 构建每个平台
for target in "${TARGETS[@]}"; do
  echo "Building for $target..."
  
  # 安装目标（如果需要）
  rustup target add "$target" 2>/dev/null || true
  
  # 构建
  cargo build --release --target "$target"
  
  # 复制到 bin 目录
  case "$target" in
    *linux*)
      cp "target/$target/release/spring-lsp" "vscode/bin/spring-lsp-linux-x64"
      ;;
    *darwin*)
      if [[ "$target" == *"aarch64"* ]]; then
        cp "target/$target/release/spring-lsp" "vscode/bin/spring-lsp-darwin-arm64"
      else
        cp "target/$target/release/spring-lsp" "vscode/bin/spring-lsp-darwin-x64"
      fi
      ;;
    *windows*)
      cp "target/$target/release/spring-lsp.exe" "vscode/bin/spring-lsp-win32-x64.exe"
      ;;
  esac
done

echo "✅ All platforms built successfully!"
echo "📦 Binaries are in vscode/bin/"
ls -lh vscode/bin/
```

#### `scripts/package-extension.sh`

```bash
#!/bin/bash

# 打包 VSCode 扩展

set -e

echo "📦 Packaging Spring LSP extension..."

# 1. 构建语言服务器（如果需要）
if [ ! -d "bin" ] || [ -z "$(ls -A bin)" ]; then
  echo "⚠️  No server binaries found. Building..."
  ./scripts/build-server.sh
fi

# 2. 编译 TypeScript
echo "🔨 Compiling TypeScript..."
npm run compile

# 3. 运行验证
echo "✅ Verifying configuration..."
npm run verify

# 4. 打包扩展
echo "📦 Creating VSIX package..."
vsce package

echo "✅ Extension packaged successfully!"
ls -lh *.vsix
```

#### `scripts/download-server.sh`

```bash
#!/bin/bash

# 从 GitHub Releases 下载预构建的语言服务器

set -e

VERSION=${1:-latest}

echo "📥 Downloading Spring LSP server binaries (version: $VERSION)..."

# 创建 bin 目录
mkdir -p bin

# GitHub Release URL
if [ "$VERSION" = "latest" ]; then
  RELEASE_URL="https://api.github.com/repos/spring-rs/spring-lsp/releases/latest"
else
  RELEASE_URL="https://api.github.com/repos/spring-rs/spring-lsp/releases/tags/$VERSION"
fi

# 获取下载链接
echo "Fetching release info..."
ASSETS=$(curl -s "$RELEASE_URL" | grep "browser_download_url" | cut -d '"' -f 4)

# 下载每个平台的二进制文件
for asset in $ASSETS; do
  filename=$(basename "$asset")
  echo "Downloading $filename..."
  curl -L -o "bin/$filename" "$asset"
done

# 设置执行权限
chmod +x bin/spring-lsp-*

echo "✅ Download complete!"
ls -lh bin/
```

### 3. 更新 package.json

```json
{
  "scripts": {
    "vscode:prepublish": "npm run compile && npm run build:server",
    "compile": "tsc -p ./",
    "watch": "tsc -watch -p ./",
    "build:server": "bash scripts/build-server.sh",
    "download:server": "bash scripts/download-server.sh",
    "package": "bash scripts/package-extension.sh",
    "package:quick": "vsce package",
    "clean": "bash scripts/clean.sh",
    "verify": "node scripts/verify.js"
  }
}
```

### 4. 更新 .vscodeignore

确保二进制文件被包含在扩展包中：

```
# .vscodeignore

# 源代码（不打包）
src/**
test/**
.vscode/**
.vscode-test/**
tsconfig.json
.eslintrc.json
.prettierrc.json

# 构建脚本（不打包）
scripts/**

# 文档（不打包，除了 README 和 CHANGELOG）
*.md
!README.md
!CHANGELOG.md

# 其他
.gitignore
.gitattributes
**/*.map
**/*.ts

# 重要：不要忽略 bin 目录！
# bin/ 目录应该被包含
!bin/**
```

### 5. 更新 LanguageClientManager

已经实现了正确的查找逻辑，但需要根据平台选择正确的二进制文件：

```typescript
private async findServerExecutable(): Promise<string | undefined> {
  // 1. 检查配置中指定的路径
  const config = vscode.workspace.getConfiguration('spring-rs');
  const configPath = config.get<string>('serverPath');

  if (configPath) {
    if (fs.existsSync(configPath)) {
      return configPath;
    } else {
      this.outputChannel.appendLine(
        `Configured server path does not exist: ${configPath}`
      );
    }
  }

  // 2. 检查扩展目录中的二进制文件（根据平台选择）
  const extensionPath = this.context.extensionPath;
  const binaryName = this.getPlatformBinaryName();
  const binaryPath = path.join(extensionPath, 'bin', binaryName);

  if (fs.existsSync(binaryPath)) {
    // 确保有执行权限（Unix 系统）
    if (process.platform !== 'win32') {
      try {
        fs.chmodSync(binaryPath, 0o755);
      } catch (error) {
        this.outputChannel.appendLine(`Failed to set execute permission: ${error}`);
      }
    }
    return binaryPath;
  }

  // 3. 检查系统 PATH
  const pathResult = await this.findInPath('spring-lsp');
  if (pathResult) {
    return pathResult;
  }

  return undefined;
}

/**
 * 获取当前平台的二进制文件名
 */
private getPlatformBinaryName(): string {
  const platform = process.platform;
  const arch = process.arch;

  if (platform === 'win32') {
    return 'spring-lsp-win32-x64.exe';
  } else if (platform === 'darwin') {
    return arch === 'arm64' 
      ? 'spring-lsp-darwin-arm64' 
      : 'spring-lsp-darwin-x64';
  } else {
    return 'spring-lsp-linux-x64';
  }
}
```

## 打包流程

### 开发环境打包

```bash
cd spring-lsp/vscode

# 方法 1: 完整构建（推荐）
npm run package

# 方法 2: 使用预构建的服务器
npm run download:server
npm run package:quick

# 方法 3: 快速打包（不包含服务器，用于测试）
npm run package:quick
```

### CI/CD 自动化

#### GitHub Actions 示例

```yaml
# .github/workflows/release.yml

name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build-server:
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            artifact: spring-lsp-linux-x64
          - os: macos-latest
            target: x86_64-apple-darwin
            artifact: spring-lsp-darwin-x64
          - os: macos-latest
            target: aarch64-apple-darwin
            artifact: spring-lsp-darwin-arm64
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            artifact: spring-lsp-win32-x64.exe

    runs-on: ${{ matrix.os }}

    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: ${{ matrix.target }}
      
      - name: Build
        run: cargo build --release --target ${{ matrix.target }}
      
      - name: Upload artifact
        uses: actions/upload-artifact@v3
        with:
          name: ${{ matrix.artifact }}
          path: target/${{ matrix.target }}/release/spring-lsp*

  package-extension:
    needs: build-server
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: 18
      
      - name: Download all artifacts
        uses: actions/download-artifact@v3
        with:
          path: vscode/bin
      
      - name: Install dependencies
        run: |
          cd vscode
          npm install
      
      - name: Compile
        run: |
          cd vscode
          npm run compile
      
      - name: Package
        run: |
          cd vscode
          npm run package:quick
      
      - name: Upload VSIX
        uses: actions/upload-artifact@v3
        with:
          name: spring-rs-extension
          path: vscode/*.vsix
      
      - name: Create Release
        uses: softprops/action-gh-release@v1
        with:
          files: vscode/*.vsix
```

## 用户安装方式

### 捆绑模式（推荐）

用户只需：
1. 在 VSCode Marketplace 搜索 "Spring RS"
2. 点击安装
3. 立即使用 ✅

### 独立安装模式

如果用户想使用自己编译的版本：

1. 安装扩展
2. 安装语言服务器：
   ```bash
   cargo install spring-lsp
   ```
3. 配置路径（可选）：
   ```json
   {
     "spring-rs.serverPath": "/path/to/spring-lsp"
   }
   ```

## 版本管理

### 版本号同步

保持扩展和语言服务器版本一致：

```json
// vscode/package.json
{
  "version": "0.1.0"
}
```

```toml
# Cargo.toml
[package]
version = "0.1.0"
```

### 更新流程

1. 更新语言服务器代码
2. 更新扩展代码
3. 同步更新两个 `version` 字段
4. 构建并打包
5. 发布到 Marketplace

## 测试

### 测试捆绑的服务器

```bash
# 1. 打包扩展
npm run package

# 2. 安装 VSIX
code --install-extension spring-rs-0.1.0.vsix

# 3. 重启 VSCode

# 4. 打开一个 spring-rs 项目

# 5. 检查 Output 面板
# View → Output → 选择 "Spring LSP"
# 应该看到：
# Found Spring LSP server at: /path/to/extension/bin/spring-lsp-xxx
```

### 测试独立安装

```bash
# 1. 安装语言服务器
cargo install --path .

# 2. 验证安装
spring-lsp --version

# 3. 安装扩展（不包含服务器）
npm run package:quick
code --install-extension spring-rs-0.1.0.vsix

# 4. 重启 VSCode

# 5. 检查是否使用系统 PATH 中的服务器
```

## 故障排查

### 服务器未找到

**症状**: Output 面板显示 "Language server not found"

**解决方案**:
1. 检查 `bin/` 目录是否存在
2. 检查二进制文件是否有执行权限
3. 手动配置路径：
   ```json
   {
     "spring-rs.serverPath": "/path/to/spring-lsp"
   }
   ```

### 权限问题（macOS/Linux）

**症状**: "Permission denied"

**解决方案**:
```bash
chmod +x ~/.vscode/extensions/spring-rs.spring-rs-*/bin/spring-lsp-*
```

### 平台不匹配

**症状**: "Exec format error"

**解决方案**: 确保下载了正确平台的二进制文件

## 推荐配置

### 开发者

```json
{
  "spring-rs.serverPath": "/path/to/dev/spring-lsp/target/release/spring-lsp",
  "spring-rs.trace.server": "verbose"
}
```

### 普通用户

不需要任何配置，开箱即用！

## 总结

**推荐方案**: 捆绑模式

**优点**:
- ✅ 最佳用户体验
- ✅ 版本一致性
- ✅ 无需额外配置

**实现步骤**:
1. 创建构建脚本
2. 更新 package.json
3. 配置 CI/CD
4. 测试打包
5. 发布到 Marketplace

---

**让用户开箱即用！** 🚀
