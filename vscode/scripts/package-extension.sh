#!/bin/bash

# 打包 VSCode 扩展

set -e

echo "📦 Packaging Spring LSP extension..."

# 进入 vscode 目录
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/.."

# 1. 检查是否有语言服务器二进制文件
if [ ! -d "bin" ] || [ -z "$(ls -A bin 2>/dev/null)" ]; then
  echo ""
  echo "⚠️  No server binaries found in bin/ directory."
  echo ""
  echo "Options:"
  echo "  1. Build all platforms: npm run build:server"
  echo "  2. Build current platform only: npm run build:server:current"
  echo "  3. Download from releases: npm run download:server"
  echo "  4. Continue without server (for testing): press Enter"
  echo ""
  read -p "Choose an option (1-4) or press Ctrl+C to cancel: " choice
  
  case $choice in
    1)
      npm run build:server
      ;;
    2)
      npm run build:server:current
      ;;
    3)
      npm run download:server
      ;;
    4)
      echo "⚠️  Continuing without server binaries..."
      ;;
    *)
      echo "❌ Invalid choice. Exiting."
      exit 1
      ;;
  esac
fi

# 2. 编译 TypeScript
echo ""
echo "🔨 Compiling TypeScript..."
npm run compile

# 3. 运行验证
echo ""
echo "✅ Verifying configuration..."
npm run verify || {
  echo "⚠️  Verification failed, but continuing..."
}

# 4. 打包扩展
echo ""
echo "📦 Creating VSIX package..."
npx vsce package

echo ""
echo "✅ Extension packaged successfully!"
echo ""
ls -lh *.vsix
echo ""
echo "To install: code --install-extension $(ls -t *.vsix | head -1)"
