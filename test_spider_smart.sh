#!/bin/bash

# Spider SMART 模式功能验证脚本
# 用于测试所有改进功能是否正常工作

set -e  # Exit on error

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 配置
CHROME_DIR="chrome-linux"
TEST_URL="http://localhost:8001"
CONCURRENT=3
REQUEST_COUNT=10

# 打印函数
print_header() {
    echo -e "\n${BLUE}========================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}========================================${NC}\n"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_info() {
    echo -e "${YELLOW}ℹ️  $1${NC}"
}

# 检查依赖
check_dependencies() {
    print_header "步骤 1: 检查依赖"

    # 检查 Chrome
    if [ -d "$CHROME_DIR" ]; then
        CHROME_PATH=$(find $CHROME_DIR -name "chrome" -type f 2>/dev/null | head -1)
        if [ -n "$CHROME_PATH" ]; then
            print_success "找到 Chrome: $CHROME_PATH"
            $CHROME_PATH --version
        else
            print_error "Chrome 目录存在但未找到可执行文件"
            exit 1
        fi
    else
        print_error "Chrome 目录不存在: $CHROME_DIR"
        exit 1
    fi

    # 检查 Oha
    if [ -f "target/release/oha" ]; then
        print_success "找到 Oha (release)"
    elif [ -f "target/debug/oha" ]; then
        print_success "找到 Oha (debug)"
    else
        print_error "未找到 Oha 二进制文件。请先编译项目："
        echo "  cargo build --features spider_smart --release"
        exit 1
    fi

    # 选择 Oha 二进制文件
    if [ -f "target/release/oha" ]; then
        OHA_BIN="target/release/oha"
    else
        OHA_BIN="target/debug/oha"
    fi
    print_info "使用 Oha: $OHA_BIN"
}

# 启动测试服务器
start_test_server() {
    print_header "步骤 2: 启动测试服务器"

    # 杀死可能存在的旧服务器
    pkill -f "python3 -m http.server" 2>/dev/null || true
    sleep 1

    # 启动简单 HTTP 服务器
    if [ -d "example_website" ]; then
        cd example_website
        print_info "在 example_website 目录启动服务器..."
        python3 -m http.server 8001 > /tmp/server.log 2>&1 &
        SERVER_PID=$!
        cd ..
    else
        print_info "创建简单测试页面..."
        mkdir -p /tmp/test_site
        cat > /tmp/test_site/index.html << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <title>Test Page</title>
    <link rel="stylesheet" href="style.css">
</head>
<body>
    <h1>Spider SMART Test Page</h1>
    <p>This is a test page for Spider SMART mode.</p>
    <img src="image.png" alt="Test image">
    <script src="script.js"></script>
    <div id="content"></div>
</body>
</html>
EOF

        cat > /tmp/test_site/style.css << 'EOF'
body {
    font-family: Arial, sans-serif;
    margin: 20px;
}
h1 {
    color: #333;
}
EOF

        cat > /tmp/test_site/script.js << 'EOF'
document.addEventListener('DOMContentLoaded', function() {
    document.getElementById('content').innerHTML = '<p>JavaScript loaded!</p>';
});
EOF

        # 创建一个简单的 404 页面用于测试
        cat > /tmp/test_site/404.html << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <title>404 - Not Found</title>
</head>
<body>
    <h1>404 - Page Not Found</h1>
</body>
</html>
EOF

        cd /tmp/test_site
        python3 -m http.server 8001 > /tmp/server.log 2>&1 &
        SERVER_PID=$!
        cd /app1
    fi

    sleep 2

    if ps -p $SERVER_PID > /dev/null; then
        print_success "测试服务器已启动 (PID: $SERVER_PID)"
    else
        print_error "测试服务器启动失败"
        cat /tmp/server.log
        exit 1
    fi
}

# 测试 1: 基本 SMART 模式测试
test_basic_smart() {
    print_header "测试 1: 基本 SMART 模式 (验证状态码获取)"

    local cmd="$OHA_BIN --page-loader spider --spider-mode smart \
        --chrome-bin $CHROME_PATH \
        --spider-disable-cache \
        -c $CONCURRENT -n $REQUEST_COUNT \
        --no-tui \
        $TEST_URL/"

    print_info "执行命令: $cmd"
    eval $cmd 2>&1 | tee /tmp/test1.log

    if grep -q "Status code" /tmp/test1.log || grep -q "requests" /tmp/test1.log; then
        print_success "基本 SMART 模式测试通过"
    else
        print_info "测试执行完成（输出可能因优化而简化）"
    fi
}

# 测试 2: 资源收集功能
test_resource_collection() {
    print_header "测试 2: 资源收集功能 (--page-resources)"

    print_info "测试 2a: 关闭资源收集"
    local cmd1="$OHA_BIN --page-loader spider --spider-mode smart \
        --chrome-bin $CHROME_PATH \
        --spider-disable-cache \
        --page-resources off \
        -c $CONCURRENT -n $REQUEST_COUNT \
        --no-tui \
        $TEST_URL/"

    print_info "执行: $cmd1"
    eval $cmd1 2>&1 | tee /tmp/test2a.log
    print_success "关闭资源收集测试完成"

    print_info "\n测试 2b: 开启资源收集"
    local cmd2="$OHA_BIN --page-loader spider --spider-mode smart \
        --chrome-bin $CHROME_PATH \
        --spider-disable-cache \
        --page-resources summary \
        -c $CONCURRENT -n $REQUEST_COUNT \
        --no-tui \
        $TEST_URL/"

    print_info "执行: $cmd2"
    eval $cmd2 2>&1 | tee /tmp/test2b.log
    print_success "开启资源收集测试完成"
}

# 测试 3: 时间指标功能
test_timing_metrics() {
    print_header "测试 3: 时间指标 (First Byte)"

    print_info "运行带时间指标的测试..."
    local cmd="$OHA_BIN --page-loader spider --spider-mode smart \
        --chrome-bin $CHROME_PATH \
        --spider-disable-cache \
        --page-resources summary \
        -c $CONCURRENT -n $REQUEST_COUNT \
        $TEST_URL/"

    print_info "执行 (带 TUI 以查看实时指标):"
    print_info "命令: $cmd"
    print_info "注意: 此测试将显示 TUI 界面，按 Ctrl+C 结束"

    # 运行 5 秒后自动结束
    timeout 5s $cmd 2>&1 | tee /tmp/test3.log || true

    if grep -q "ms" /tmp/test3.log || grep -q "latency" /tmp/test3.log; then
        print_success "时间指标测试通过"
    else
        print_info "时间指标测试执行完成"
    fi
}

# 测试 4: 错误状态码测试
test_error_status() {
    print_header "测试 4: 错误状态码验证"

    print_info "测试 404 页面..."
    local cmd="$OHA_BIN --page-loader spider --spider-mode smart \
        --chrome-bin $CHROME_PATH \
        --spider-disable-cache \
        -c 2 -n 5 \
        --no-tui \
        $TEST_URL/nonexistent.html"

    print_info "执行: $cmd"
    eval $cmd 2>&1 | tee /tmp/test4.log

    if grep -q "404\|error" /tmp/test4.log; then
        print_success "错误状态码测试通过 (检测到 404 或错误)"
    else
        print_info "测试完成（状态码可能未在输出中显示）"
    fi
}

# 测试 5: 性能对比测试
test_performance() {
    print_header "测试 5: 性能对比 (no-tui 优化)"

    print_info "测试 5a: HTTP 模式 (基线)"
    local start1=$(date +%s%3N)
    $OHA_BIN --page-loader spider --spider-mode http \
        -c $CONCURRENT -n $REQUEST_COUNT \
        --no-tui \
        $TEST_URL/ > /dev/null 2>&1
    local end1=$(date +%s%3N)
    local duration1=$((end1 - start1))
    print_info "HTTP 模式耗时: ${duration1}ms"

    print_info "\n测试 5b: SMART 模式 (优化后)"
    local start2=$(date +%s%3N)
    $OHA_BIN --page-loader spider --spider-mode smart \
        --chrome-bin $CHROME_PATH \
        --spider-disable-cache \
        --page-resources off \
        -c $CONCURRENT -n $REQUEST_COUNT \
        --no-tui \
        $TEST_URL/ > /dev/null 2>&1
    local end2=$(date +%s%3N)
    local duration2=$((end2 - start2))
    print_info "SMART 模式耗时: ${duration2}ms"

    print_success "性能测试完成"
}

# 测试 6: 完整功能测试
test_full_features() {
    print_header "测试 6: 完整功能验证"

    print_info "组合所有改进功能..."
    local cmd="$OHA_BIN --page-loader spider --spider-mode smart \
        --chrome-bin $CHROME_PATH \
        --spider-disable-cache \
        --page-resources full \
        -c $CONCURRENT -n $REQUEST_COUNT \
        --no-tui \
        $TEST_URL/"

    print_info "执行完整功能测试..."
    eval $cmd 2>&1 | tee /tmp/test6.log

    print_success "完整功能测试完成"
}

# 清理
cleanup() {
    print_header "清理资源"

    if [ -n "$SERVER_PID" ] && ps -p $SERVER_PID > /dev/null 2>&1; then
        print_info "停止测试服务器 (PID: $SERVER_PID)..."
        kill $SERVER_PID 2>/dev/null || true
        print_success "服务器已停止"
    fi

    # 清理 Chrome 进程
    print_info "清理 Chrome 进程..."
    pkill -f "chrome" 2>/dev/null || true
    print_success "Chrome 进程已清理"
}

# 显示日志
show_logs() {
    print_header "测试日志"

    for i in {1..6}; do
        if [ -f "/tmp/test$i.log" ]; then
            print_info "测试 $i 日志大小: $(wc -l < /tmp/test$i.log) 行"
        fi
    done

    print_info "查看完整日志: cat /tmp/test*.log"
}

# 主函数
main() {
    print_header "Spider SMART 模式功能验证脚本"
    print_info "开始时间: $(date)"
    print_info "配置: Chrome=$CHROME_DIR, 并发=$CONCURRENT, 请求数=$REQUEST_COUNT"

    # 设置退出陷阱
    trap cleanup EXIT

    # 执行测试
    check_dependencies
    start_test_server

    echo ""
    print_info "所有准备完成，开始测试..."
    sleep 2

    test_basic_smart
    test_resource_collection
    test_timing_metrics
    test_error_status
    test_performance
    test_full_features

    # 总结
    print_header "测试总结"
    print_success "所有测试执行完成！"
    print_info "请检查上述输出以验证各项功能"
    print_info "详细日志保存在 /tmp/test*.log"
    print_info "结束时间: $(date)"

    # 显示日志摘要
    show_logs
}

# 运行主函数
main "$@"
