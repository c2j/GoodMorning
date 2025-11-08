#!/bin/bash

# Spider SMART 快速验证脚本
# 简化版本，用于快速检查核心功能

set -e

# 颜色
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}================================${NC}"
echo -e "${BLUE}Spider SMART 快速验证${NC}"
echo -e "${BLUE}================================${NC}\n"

# 1. 检查构建
echo -e "${YELLOW}[1/5] 检查构建...${NC}"
if [ -f "target/release/oha" ]; then
    echo -e "${GREEN}✅ 找到 oha (release)${NC}"
    OHA="target/release/oha"
elif [ -f "target/debug/oha" ]; then
    echo -e "${GREEN}✅ 找到 oha (debug)${NC}"
    OHA="target/debug/oha"
else
    echo "❌ 未找到 oha，请先编译: cargo build --features spider_smart --release"
    exit 1
fi

# 2. 检查 Chrome
echo -e "\n${YELLOW}[2/5] 检查 Chrome...${NC}"
if [ -d "chrome-linux" ]; then
    CHROME=$(find chrome-linux -name "chrome" -type f 2>/dev/null | head -1)
    if [ -n "$CHROME" ]; then
        echo -e "${GREEN}✅ 找到 Chrome: $CHROME${NC}"
    else
        echo "❌ chrome-linux 目录存在但未找到可执行文件"
        exit 1
    fi
else
    echo "❌ 未找到 chrome-linux 目录"
    exit 1
fi

# 3. 启动测试服务器
echo -e "\n${YELLOW}[3/5] 启动测试服务器...${NC}"
pkill -f "python3 -m http.server" 2>/dev/null || true
sleep 1

# 创建简单测试页面
mkdir -p /tmp/test_site
cat > /tmp/test_site/index.html << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <title>Spider Test</title>
    <style>body{font-family:sans-serif;margin:20px}</style>
</head>
<body>
    <h1>Spider SMART Test Page</h1>
    <p>Testing page resources and timing</p>
    <img src="data:image/svg+xml,%3Csvg%20xmlns='http://www.w3.org/2000/svg'%20width='100'%20height='100'%3E%3Crect%20fill='blue'/%3E%3C/svg%3E" alt="test">
    <script>console.log('test');</script>
</body>
</html>
EOF

cd /tmp/test_site
python3 -m http.server 8001 > /tmp/server.log 2>&1 &
SERVER_PID=$!
cd /app1
sleep 2

if ps -p $SERVER_PID > /dev/null; then
    echo -e "${GREEN}✅ 测试服务器运行中 (PID: $SERVER_PID)${NC}"
else
    echo "❌ 服务器启动失败"
    exit 1
fi

# 4. 运行测试
echo -e "\n${YELLOW}[4/5] 运行 Spider SMART 测试...${NC}"

echo -e "\n${BLUE}测试 A: 基础 SMART 模式${NC}"
$OHA --page-loader spider --spider-mode smart \
    --chrome-bin "$CHROME" \
    --spider-disable-cache \
    -c 2 -n 5 \
    --no-tui \
    http://localhost:8001/ 2>&1 | head -20

echo -e "\n${BLUE}测试 B: 带资源收集${NC}"
$OHA --page-loader spider --spider-mode smart \
    --chrome-bin "$CHROME" \
    --spider-disable-cache \
    --page-resources summary \
    -c 2 -n 5 \
    --no-tui \
    http://localhost:8001/ 2>&1 | head -20

echo -e "\n${BLUE}测试 C: TUI 模式 (5秒)${NC}"
timeout 5s $OHA --page-loader spider --spider-mode smart \
    --chrome-bin "$CHROME" \
    --spider-disable-cache \
    --page-resources summary \
    -c 2 -n 10 \
    http://localhost:8001/ 2>&1 || true

# 5. 清理
echo -e "\n${YELLOW}[5/5] 清理...${NC}"
kill $SERVER_PID 2>/dev/null || true
pkill -f "chrome" 2>/dev/null || true
sleep 1

echo -e "\n${GREEN}✅ 快速验证完成！${NC}"
echo -e "\n${BLUE}要运行完整测试，请执行:${NC}"
echo -e "  ./test_spider_smart.sh"
