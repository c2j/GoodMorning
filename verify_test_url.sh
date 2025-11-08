#!/bin/bash

# 验证测试 URL 可访问性和结构

set -e

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}================================${NC}"
echo -e "${BLUE}验证测试 URL 可访问性${NC}"
echo -e "${BLUE}================================${NC}\n"

# 测试 URL
TEST_URL="http://httpbin.org/html"

echo -e "${YELLOW}正在检查 URL: $TEST_URL${NC}\n"

# 1. 检查网络连接
echo -e "${BLUE}[1/4] 检查网络连接...${NC}"
if curl -s --max-time 5 "$TEST_URL" > /tmp/test_page.html; then
    echo -e "${GREEN}✅ URL 可访问${NC}"
    SIZE=$(wc -c < /tmp/test_page.html)
    echo "   页面大小: $SIZE 字节"
else
    echo -e "${RED}❌ 无法访问 URL${NC}"
    echo "   请检查网络连接或防火墙设置"
    exit 1
fi

# 2. 分析页面结构
echo -e "\n${BLUE}[2/4] 分析页面结构...${NC}"
if grep -q "<!DOCTYPE html>" /tmp/test_page.html; then
    echo -e "${GREEN}✅ HTML 文档结构正确${NC}"
fi

if grep -q "<img" /tmp/test_page.html; then
    IMG_COUNT=$(grep -o "<img" /tmp/test_page.html | wc -l)
    echo -e "${GREEN}✅ 包含图片元素 ($IMG_COUNT 个)${NC}"
fi

if grep -q "<script" /tmp/test_page.html; then
    SCRIPT_COUNT=$(grep -o "<script" /tmp/test_page.html | wc -l)
    echo -e "${GREEN}✅ 包含脚本元素 ($SCRIPT_COUNT 个)${NC}"
fi

if grep -q "<link" /tmp/test_page.html; then
    LINK_COUNT=$(grep -o "<link" /tmp/test_page.html | wc -l)
    echo -e "${GREEN}✅ 包含链接元素 ($LINK_COUNT 个)${NC}"
fi

# 3. 检查 HTTP 头信息
echo -e "\n${BLUE}[3/4] 检查 HTTP 响应头...${NC}"
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" --max-time 5 "$TEST_URL")
if [ "$HTTP_CODE" = "200" ]; then
    echo -e "${GREEN}✅ HTTP 状态码: $HTTP_CODE${NC}"
else
    echo -e "${YELLOW}⚠️  HTTP 状态码: $HTTP_CODE (期望 200)${NC}"
fi

CONTENT_TYPE=$(curl -s -I --max-time 5 "$TEST_URL" | grep -i "content-type" | head -1 | cut -d' ' -f2-)
echo "   Content-Type: ${CONTENT_TYPE:-unknown}"

# 4. 性能测试
echo -e "\n${BLUE}[4/4] 性能测试...${NC}"
echo "   执行 5 次请求..."

TOTAL_TIME=0
for i in {1..5}; do
    START=$(date +%s%3N)
    HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" --max-time 10 "$TEST_URL")
    END=$(date +%s%3N)
    DURATION=$((END - START))
    TOTAL_TIME=$((TOTAL_TIME + DURATION))
    echo "   请求 $i: ${DURATION}ms (状态: $HTTP_CODE)"
done

AVG_TIME=$((TOTAL_TIME / 5))
echo -e "${GREEN}✅ 平均响应时间: ${AVG_TIME}ms${NC}"

# 分析页面内容
echo -e "\n${BLUE}页面内容摘要:${NC}"
head -20 /tmp/test_page.html | sed 's/^/  /'

# 生成测试命令
echo -e "\n${GREEN}================================${NC}"
echo -e "${GREEN}生成测试命令${NC}"
echo -e "${GREEN}================================${NC}\n"

# 检查是否有 oha
if [ -f "target/release/oha" ]; then
    OHA="target/release/oha"
    echo -e "${GREEN}✅ 找到 oha (release)${NC}"
elif [ -f "target/debug/oha" ]; then
    OHA="target/debug/oha"
    echo -e "${GREEN}✅ 找到 oha (debug)${NC}"
else
    OHA="oha"
    echo -e "${YELLOW}⚠️  未找到编译的 oha，假设在 PATH 中${NC}"
fi

# 检查 Chrome
if [ -d "chrome-linux" ]; then
    CHROME="chrome-linux/chrome"
    echo -e "${GREEN}✅ 找到 Chrome: $CHROME${NC}"
else
    echo -e "${YELLOW}⚠️  未找到 chrome-linux 目录${NC}"
    CHROME="chromium"
fi

# 生成命令
echo -e "\n${BLUE}基本 SMART 测试命令:${NC}"
cat << EOF
$OHA --page-loader spider --spider-mode smart \\
  --chrome-bin $CHROME \\
  --spider-disable-cache \\
  -c 2 -n 5 --no-tui \\
  $TEST_URL
EOF

echo -e "\n${BLUE}带资源收集的测试命令:${NC}"
cat << EOF
$OHA --page-loader spider --spider-mode smart \\
  --chrome-bin $CHROME \\
  --spider-disable-cache \\
  --page-resources summary \\
  -c 2 -n 5 --no-tui \\
  $TEST_URL
EOF

echo -e "\n${BLUE}TUI 模式测试命令:${NC}"
cat << EOF
$OHA --page-loader spider --spider-mode smart \\
  --chrome-bin $CHROME \\
  --spider-disable-cache \\
  --page-resources summary \\
  -c 2 -n 10 \\
  $TEST_URL
EOF

# 清理
rm -f /tmp/test_page.html

echo -e "\n${GREEN}✅ URL 验证完成！${NC}"
echo -e "\n要运行实际测试，请先编译 oha:"
echo -e "  ${BLUE}cargo build --features spider_smart --release${NC}"
echo -e "\n然后运行测试脚本:"
echo -e "  ${BLUE}./quick_test.sh${NC}"
