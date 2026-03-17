#!/bin/bash
# SMA-OS 版本号管理脚本
# 用法: ./scripts/update-version.sh <new_version>
# 例如: ./scripts/update-version.sh 1.1.1

set -e

if [ $# -eq 0 ]; then
    echo "用法: $0 <new_version>"
    echo "例如: $0 1.1.1"
    exit 1
fi

NEW_VERSION="$1"
VERSION_FILE="VERSION"

# 验证版本号格式
if ! [[ $NEW_VERSION =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "错误: 版本号格式不正确。应使用语义化版本号 (如 1.1.0)"
    exit 1
fi

echo "正在更新版本号到 v${NEW_VERSION}..."

# ============================================
# 1. 更新 VERSION 文件
# ============================================
echo "$NEW_VERSION" > "$VERSION_FILE"
echo "✓ 已更新 $VERSION_FILE"

# ============================================
# 2. 更新 Rust Cargo.toml 文件
# ============================================
# 根 workspace
if [ -f "control-plane/Cargo.toml" ]; then
    sed -i "s/^version = \"[^\"]*\"/version = \"${NEW_VERSION}\"/" control-plane/Cargo.toml 2>/dev/null || true
fi

# 各个 crate
for cargo_file in control-plane/*/Cargo.toml control-plane/xtask/Cargo.toml; do
    if [ -f "$cargo_file" ]; then
        sed -i "s/^version = \"[^\"]*\"/version = \"${NEW_VERSION}\"/" "$cargo_file"
        echo "✓ 已更新 $cargo_file"
    fi
done

# ============================================
# 3. 更新 Go go.mod 文件 (可选，Go 通常不强制要求版本号)
# ============================================
# Go 模块通常不更新版本号，除非发布

# ============================================
# 4. 更新 package.json
# ============================================
if [ -f "observability-ui/web-dashboard/package.json" ]; then
    sed -i "s/\"version\": \"[^\"]*\"/\"version\": \"${NEW_VERSION}\"/" observability-ui/web-dashboard/package.json
    echo "✓ 已更新 package.json"
fi

# ============================================
# 5. 更新文档
# ============================================
# README.md - 标题
sed -i "s/^# SMA-OS v[0-9]\+\.[0-9]\+\.[0-9]\+$/# SMA-OS v${NEW_VERSION}/" README.md 2>/dev/null || true

# RELEASE_NOTES.md - 如果创建新版本
if [ -f "RELEASE_NOTES.md" ]; then
    sed -i "1s/^# SMA-OS Release v.*/# SMA-OS Release v${NEW_VERSION}/" RELEASE_NOTES.md 2>/dev/null || true
fi

echo ""
echo "=========================================="
echo "版本号更新完成: v${NEW_VERSION}"
echo "=========================================="
echo ""
echo "请检查以下文件:"
echo "  - VERSION"
echo "  - control-plane/Cargo.toml"
echo "  - control-plane/*/Cargo.toml"
echo "  - observability-ui/web-dashboard/package.json"
echo "  - README.md"
echo ""
echo "然后提交更改:"
echo "  git add -A"
echo "  git commit -m \"chore: bump version to v${NEW_VERSION}\""
echo "  git tag -a v${NEW_VERSION} -m \"SMA-OS v${NEW_VERSION}\""
echo "  git push origin main --tags"
