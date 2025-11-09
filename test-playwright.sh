#!/bin/bash
# OHA Playwright 功能测试脚本

set -e

echo "========================================="
echo "  OHA Playwright 功能测试"
echo "========================================="
echo ""

# 检查构建
echo "步骤 1: 检查构建配置..."
if grep -q "playwright = { git" Cargo.toml; then
    echo "✅ Cargo.toml 已配置 Playwright 依赖"
else
    echo "❌ Cargo.toml 未配置 Playwright 依赖"
    echo "请运行: bash restore-playwright.sh"
    exit 1
fi

# 检查模块
echo ""
echo "步骤 2: 检查模块文件..."
REQUIRED_FILES=(
    "src/page_test/mod.rs"
    "src/page_test/cli.rs"
    "src/page_test/types.rs"
    "src/page_test/client.rs"
    "src/page_test/recorder.rs"
)

ALL_EXIST=true
for file in "${REQUIRED_FILES[@]}"; do
    if [ -f "$file" ]; then
        echo "✅ $file"
    else
        echo "❌ $file 不存在"
        ALL_EXIST=false
    fi
done

if [ "$ALL_EXIST" = false ]; then
    exit 1
fi

# 检查 lib.rs 集成
echo ""
echo "步骤 3: 检查 lib.rs 集成..."
if grep -q "cfg(feature = \"playwright\")" src/lib.rs && grep -q "mod page_test" src/lib.rs; then
    echo "✅ lib.rs 已集成 page_test 模块"
else
    echo "❌ lib.rs 集成不完整"
    exit 1
fi

echo ""
echo "========================================="
echo "  所有检查通过!"
echo "========================================="
echo ""
echo "下一步:"
echo "  1. 构建项目: cargo build --release --features playwright"
echo "  2. 测试录制: oha --pw-record --pw-output test.json <url>"
echo "  3. 查看文档: cat docs/recording-guide.md"
echo ""
