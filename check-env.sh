#!/bin/bash
echo "=== Playwright 环境检查 ==="
echo ""

# Check if binary exists
echo "1. 检查二进制文件..."
if [ -f "./target/release/oha" ]; then
    echo "   ✅ oha 二进制文件存在"
else
    echo "   ❌ oha 二进制文件不存在"
    echo "   运行: cargo build --release --features playwright"
    exit 1
fi

# Check help
echo ""
echo "2. 检查 Playwright 参数..."
if ./target/release/oha --help 2>&1 | grep -q "pw-record"; then
    echo "   ✅ Playwright 参数已启用"
else
    echo "   ❌ Playwright 参数未找到"
    exit 1
fi

# Check playwright installation
echo ""
echo "3. 检查 Playwright 安装..."
if command -v playwright &> /dev/null; then
    echo "   ✅ Playwright CLI 已安装"
    echo ""
    echo "   已安装的浏览器:"
    if [ -d "$HOME/.cache/ms-playwright" ]; then
        ls -1 "$HOME/.cache/ms-playwright" 2>/dev/null | sed 's/^/     - /' || echo "     未找到浏览器"
    else
        echo "     未找到浏览器缓存"
    fi
else
    echo "   ⚠️  Playwright CLI 未安装"
    echo "   安装命令: npm install -g playwright"
fi

# Check system dependencies
echo ""
echo "4. 检查系统依赖..."
MISSING_DEPS=()
for lib in libglib2.0-0 libnss3 libatk1.0-0 libatk-bridge2.0-0 libcups2 libdrm2 libxkbcommon0 libxcomposite1 libxdamage1 libxfixes3 libxrandr2 libgbm1 libasound2; do
    if ! dpkg -l | grep -q "^ii  $lib "; then
        MISSING_DEPS+=("$lib")
    fi
done

if [ ${#MISSING_DEPS[@]} -eq 0 ]; then
    echo "   ✅ 核心系统依赖已安装"
else
    echo "   ⚠️  可能缺少以下依赖:"
    for dep in "${MISSING_DEPS[@]}"; do
        echo "     - $dep"
    done
    echo "   安装命令: sudo apt-get install -y ${MISSING_DEPS[*]}"
fi

echo ""
echo "=== 检查完成 ==="
echo ""
echo "要使用录制功能:"
echo "  1. 安装浏览器: playwright install chromium"
echo "  2. 运行录制: ./target/release/oha --pw-record --pw-output scenario.json https://example.com"
echo ""
