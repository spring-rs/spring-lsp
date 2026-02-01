#!/bin/bash

# spring-lsp 发布脚本
# 用法: ./scripts/release.sh [patch|minor|major|prerelease] [alpha|beta|rc]

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 日志函数
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 检查依赖
check_dependencies() {
    log_info "检查依赖..."
    
    if ! command -v cargo &> /dev/null; then
        log_error "cargo 未安装"
        exit 1
    fi
    
    if ! command -v git &> /dev/null; then
        log_error "git 未安装"
        exit 1
    fi
    
    if ! cargo install --list | grep -q "cargo-edit"; then
        log_warning "cargo-edit 未安装，正在安装..."
        cargo install cargo-edit
    fi
    
    log_success "依赖检查完成"
}

# 检查工作目录状态
check_git_status() {
    log_info "检查 Git 状态..."
    
    if [ -n "$(git status --porcelain)" ]; then
        log_error "工作目录不干净，请先提交或暂存更改"
        git status --short
        exit 1
    fi
    
    # 检查是否在主分支
    CURRENT_BRANCH=$(git branch --show-current)
    if [ "$CURRENT_BRANCH" != "master" ] && [ "$CURRENT_BRANCH" != "main" ]; then
        log_warning "当前不在主分支 ($CURRENT_BRANCH)，确定要继续吗？ (y/N)"
        read -r response
        if [[ ! "$response" =~ ^[Yy]$ ]]; then
            log_info "发布已取消"
            exit 0
        fi
    fi
    
    log_success "Git 状态检查完成"
}

# 运行测试
run_tests() {
    log_info "运行测试..."
    
    # 格式检查
    log_info "检查代码格式..."
    cargo fmt --all -- --check
    
    # Clippy 检查
    log_info "运行 Clippy..."
    cargo clippy --all-features -- -D warnings
    
    # 单元测试
    log_info "运行单元测试..."
    cargo test --all-features
    
    # 文档测试
    log_info "运行文档测试..."
    cargo test --doc
    
    # 构建文档
    log_info "构建文档..."
    cargo doc --all-features --no-deps
    
    log_success "所有测试通过"
}

# 获取当前版本
get_current_version() {
    grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/'
}

# 更新版本
bump_version() {
    local bump_type=$1
    local prerelease_type=$2
    
    log_info "更新版本 ($bump_type)..."
    
    case $bump_type in
        patch)
            cargo set-version --bump patch
            ;;
        minor)
            cargo set-version --bump minor
            ;;
        major)
            cargo set-version --bump major
            ;;
        prerelease)
            if [ -n "$prerelease_type" ]; then
                current_version=$(get_current_version)
                if [[ "$current_version" =~ -[a-zA-Z]+\.[0-9]+$ ]]; then
                    cargo set-version --bump prerelease
                else
                    cargo set-version --bump minor
                    new_version=$(get_current_version)
                    cargo set-version "${new_version}-${prerelease_type}.1"
                fi
            else
                cargo set-version --bump prerelease
            fi
            ;;
        *)
            log_error "无效的版本类型: $bump_type"
            echo "支持的类型: patch, minor, major, prerelease"
            exit 1
            ;;
    esac
}

# 更新 CHANGELOG
update_changelog() {
    local new_version=$1
    local date=$(date +%Y-%m-%d)
    
    log_info "更新 CHANGELOG.md..."
    
    if [ -f "CHANGELOG.md" ]; then
        # 备份原文件
        cp CHANGELOG.md CHANGELOG.md.bak
        
        # 在顶部添加新版本条目
        {
            echo "## [$new_version] - $date"
            echo ""
            echo "### Added"
            echo "- 新功能和改进"
            echo ""
            echo "### Changed"
            echo "- 更新和修改"
            echo ""
            echo "### Fixed"
            echo "- Bug 修复和更正"
            echo ""
            cat CHANGELOG.md
        } > CHANGELOG.md.tmp && mv CHANGELOG.md.tmp CHANGELOG.md
        
        log_info "请编辑 CHANGELOG.md 添加具体的更改内容"
        echo "按 Enter 继续..."
        read -r
        
        # 打开编辑器（如果可用）
        if command -v $EDITOR &> /dev/null; then
            $EDITOR CHANGELOG.md
        elif command -v nano &> /dev/null; then
            nano CHANGELOG.md
        elif command -v vim &> /dev/null; then
            vim CHANGELOG.md
        fi
    else
        log_warning "CHANGELOG.md 不存在，创建新文件..."
        cat > CHANGELOG.md << EOF
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [$new_version] - $date

### Added
- Initial release of spring-lsp
- Language Server Protocol implementation for spring-rs framework
- TOML configuration support with smart completion
- Rust macro analysis and validation
- Route management and navigation

EOF
    fi
}

# 创建发布提交和标签
create_release_commit() {
    local new_version=$1
    
    log_info "创建发布提交..."
    
    git add Cargo.toml Cargo.lock CHANGELOG.md
    git commit -m "chore: release v$new_version"
    git tag -a "v$new_version" -m "Release v$new_version"
    
    log_success "创建了提交和标签 v$new_version"
}

# 推送到远程仓库
push_release() {
    local new_version=$1
    
    log_info "推送到远程仓库..."
    
    echo "即将推送以下内容到远程仓库："
    echo "  - 发布提交"
    echo "  - 标签 v$new_version"
    echo ""
    echo "这将触发 GitHub Actions 发布流程"
    echo "确定要继续吗？ (y/N)"
    
    read -r response
    if [[ "$response" =~ ^[Yy]$ ]]; then
        git push origin HEAD
        git push origin "v$new_version"
        log_success "推送完成"
        
        echo ""
        log_info "发布流程已启动！"
        echo "  - GitHub Actions: https://github.com/spring-rs/spring-lsp/actions"
        echo "  - 发布页面: https://github.com/spring-rs/spring-lsp/releases"
        echo "  - Crates.io: https://crates.io/crates/spring-lsp"
    else
        log_info "推送已取消"
        log_warning "要删除本地标签，运行: git tag -d v$new_version"
    fi
}

# 显示帮助信息
show_help() {
    echo "spring-lsp 发布脚本"
    echo ""
    echo "用法:"
    echo "  $0 [patch|minor|major|prerelease] [alpha|beta|rc]"
    echo ""
    echo "参数:"
    echo "  patch       补丁版本 (0.1.0 -> 0.1.1)"
    echo "  minor       次要版本 (0.1.0 -> 0.2.0)"
    echo "  major       主要版本 (0.1.0 -> 1.0.0)"
    echo "  prerelease  预发布版本 (0.1.0 -> 0.2.0-alpha.1)"
    echo ""
    echo "预发布类型 (仅用于 prerelease):"
    echo "  alpha       Alpha 版本"
    echo "  beta        Beta 版本"
    echo "  rc          Release Candidate"
    echo ""
    echo "示例:"
    echo "  $0 patch                    # 发布补丁版本"
    echo "  $0 minor                    # 发布次要版本"
    echo "  $0 prerelease alpha         # 发布 alpha 预发布版本"
    echo ""
}

# 主函数
main() {
    local bump_type=$1
    local prerelease_type=$2
    
    # 检查参数
    if [ -z "$bump_type" ]; then
        show_help
        exit 1
    fi
    
    if [ "$bump_type" = "help" ] || [ "$bump_type" = "--help" ] || [ "$bump_type" = "-h" ]; then
        show_help
        exit 0
    fi
    
    # 切换到项目根目录
    cd "$(dirname "$0")/.."
    
    log_info "开始 spring-lsp 发布流程..."
    echo ""
    
    # 执行发布步骤
    check_dependencies
    check_git_status
    
    local current_version=$(get_current_version)
    log_info "当前版本: $current_version"
    
    run_tests
    
    bump_version "$bump_type" "$prerelease_type"
    
    local new_version=$(get_current_version)
    log_success "版本已更新: $current_version -> $new_version"
    
    update_changelog "$new_version"
    
    create_release_commit "$new_version"
    
    push_release "$new_version"
    
    log_success "发布流程完成！"
}

# 运行主函数
main "$@"