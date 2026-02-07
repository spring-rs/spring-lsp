#!/bin/bash

# 只构建当前平台的语言服务器（用于快速开发）

set -e

echo "🔨 Building Spring LSP server for current platform..."

# 进入项目根目录
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/../.."

# 创建 bin 目录
mkdir -p vscode/bin

# 构建当前平台
echo "📦 Building..."
cargo build --release

# 检测当前平台并复制
PLATFORM=$(uname -s)
ARCH=$(uname -m)

case "$PLATFORM" in
  Linux)
    cp "target/release/spring-lsp" "vscode/bin/spring-lsp-linux-x64"
    echo "✅ Copied to vscode/bin/spring-lsp-linux-x64"
    ;;
  Darwin)
    if [ "$ARCH" = "arm64" ]; then
      cp "target/release/spring-lsp" "vscode/bin/spring-lsp-darwin-arm64"
      echo "✅ Copied to vscode/bin/spring-lsp-darwin-arm64"
    else
      cp "target/release/spring-lsp" "vscode/bin/spring-lsp-darwin-x64"
      echo "✅ Copied to vscode/bin/spring-lsp-darwin-x64"
    fi
    ;;
  MINGW*|MSYS*|CYGWIN*)
    cp "target/release/spring-lsp.exe" "vscode/bin/spring-lsp-win32-x64.exe"
    echo "✅ Copied to vscode/bin/spring-lsp-win32-x64.exe"
    ;;
  *)
    echo "❌ Unsupported platform: $PLATFORM"
    exit 1
    ;;
esac

echo ""
echo "✅ Build complete!"
ls -lh vscode/bin/
