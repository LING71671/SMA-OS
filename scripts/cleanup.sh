#!/bin/bash
# SMA-OS 项目清理脚本
# 删除临时文件和不需要的目录

echo "=== SMA-OS 项目清理 ==="
echo ""

# 要删除的临时目录
TEMP_DIRS=(
    ".trae"
    ".sisyphus"
    ".vscode"
    ".cursor"
    "A:SMA-OS.sisyphusdrafts"
    "A:SMA-OS.sisyphusplans"
    "A:SMA-OScontrol-planestate-enginesrccache"
    "A:SMA-OSmemory-busingestioninternalmetrics"
    "A:SMA-OSorchestrationconfig"
)

# 要删除的文件
TEMP_FILES=(
    "control-plane/target/.future-incompat-report.json"
    "control-plane/target/.rustc_info.json"
    "control-plane/target/CACHEDIR.TAG"
    "observability-ui/web-dashboard/.next/dev/logs/next-development.log"
)

echo "1. 删除临时目录..."
for dir in "${TEMP_DIRS[@]}"; do
    if [ -d "$dir" ]; then
        echo "   删除: $dir"
        rm -rf "$dir"
    fi
done

echo ""
echo "2. 删除临时文件..."
for file in "${TEMP_FILES[@]}"; do
    if [ -f "$file" ]; then
        echo "   删除: $file"
        rm -f "$file"
    fi
done

echo ""
echo "3. 从 git 缓存中移除已跟踪的临时文件..."
git rm -r --cached .trae/ 2>/dev/null || true
git rm -r --cached .sisyphus/ 2>/dev/null || true
git rm -r --cached .vscode/ 2>/dev/null || true
git rm -r --cached .cursor/ 2>/dev/null || true
git rm -r --cached "A:SMA-OS.sisyphusdrafts" 2>/dev/null || true
git rm -r --cached "A:SMA-OS.sisyphusplans" 2>/dev/null || true
git rm -r --cached "A:SMA-OScontrol-planestate-enginesrccache" 2>/dev/null || true
git rm -r --cached "A:SMA-OSmemory-busingestioninternalmetrics" 2>/dev/null || true
git rm -r --cached "A:SMA-OSorchestrationconfig" 2>/dev/null || true
git rm -r --cached control-plane/target/ 2>/dev/null || true

echo ""
echo "=== 清理完成 ==="
echo ""
echo "请运行以下命令提交更改："
echo "  git add -A"
echo "  git commit -m \"chore: clean up temporary files and directories\""
echo "  git push origin main"
