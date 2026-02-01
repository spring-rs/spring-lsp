#!/bin/bash
# 安装 Git hooks

HOOKS_DIR=".git/hooks"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "正在安装 Git hooks..."

# 创建 pre-commit hook
cat > "$HOOKS_DIR/pre-commit" << 'EOF'
#!/bin/sh
# Pre-commit hook: 格式化所有 Rust 代码

echo "正在运行 rustfmt..."

# 格式化所有 Rust 文件
cargo fmt --all

# 将格式化后的文件添加到暂存区
git diff --name-only --cached --diff-filter=ACM | grep '\.rs$' | xargs git add

echo "✓ 代码格式化完成"
EOF

chmod +x "$HOOKS_DIR/pre-commit"

echo "✓ Git hooks 安装完成"
echo "提示：每次提交前会自动运行 rustfmt"
