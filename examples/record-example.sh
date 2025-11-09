#!/bin/bash
# OHA Playwright 录制功能快速入门示例

set -e

echo "========================================="
echo "  OHA Playwright 录制示例"
echo "========================================="
echo ""

# 检查是否安装了 playwright 浏览器
echo "检查 Playwright 浏览器..."
if ! command -v playwright &> /dev/null; then
    echo "❌ Playwright 未安装"
    echo ""
    echo "请先安装 Playwright 浏览器："
    echo "  playwright install chromium"
    echo "  playwright install firefox"
    echo "  playwright install webkit"
    echo ""
    exit 1
fi

echo "✅ Playwright 已安装"
echo ""

# 示例 1: 基本录制
echo "========================================="
echo "示例 1: 基本页面录制"
echo "========================================="
echo ""
echo "运行命令:"
echo "  oha --pw-record --pw-output example-basic.json https://example.com"
echo ""

read -p "是否运行示例 1? (y/N): " -n 1 -r
echo ""
if [[ $REPLY =~ ^[Yy]$ ]]; then
    oha --pw-record --pw-output example-basic.json -c 1 -n 1 https://example.com 2>/dev/null || {
        echo "❌ 录制失败，可能原因："
        echo "  1. 未启用 playwright 特性"
        echo "  2. 网络连接问题"
        echo "  3. 浏览器未正确安装"
        echo ""
        echo "请确保使用以下命令构建："
        echo "  cargo build --release --features playwright"
    }
    echo ""
fi

# 示例 2: 使用指定浏览器
echo "========================================="
echo "示例 2: 使用 Firefox 录制"
echo "========================================="
echo ""
echo "运行命令:"
echo "  oha --pw-record --pw-output example-firefox.json --pw-browser firefox https://example.com"
echo ""

read -p "是否运行示例 2? (y/N): " -n 1 -r
echo ""
if [[ $REPLY =~ ^[Yy]$ ]]; then
    oha --pw-record --pw-output example-firefox.json --pw-browser firefox -c 1 -n 1 https://example.com 2>/dev/null || {
        echo "❌ 录制失败"
    }
    echo ""
fi

# 示例 3: 查看生成的 JSON
echo "========================================="
echo "示例 3: 查看生成的 JSON 文件"
echo "========================================="
echo ""

if [ -f "example-basic.json" ]; then
    echo "✅ 找到 example-basic.json"
    echo ""
    echo "内容预览："
    head -20 example-basic.json
    echo "  ..."
    echo ""
    echo "完整文件位置: $(pwd)/example-basic.json"
else
    echo "❌ 未找到 example-basic.json 文件"
fi

echo ""
echo "========================================="
echo "  使用完成"
echo "========================================="
echo ""
echo "接下来你可以："
echo "  1. 查看生成的 scenario.json 文件"
echo "  2. 将其用于回放测试（即将推出）"
echo "  3. 分析页面性能指标"
echo ""
echo "查看完整文档:"
echo "  cat docs/recording-guide.md"
echo ""
