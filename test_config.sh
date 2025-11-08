#!/bin/bash

# Spider SMART 测试配置文件
# 可以修改这些参数来自定义测试

# ==================== 基本配置 ====================

# Chrome 浏览器路径
export CHROME_BIN="chrome-linux/chrome"

# 测试 URL
export TEST_URL="http://localhost:8001"

# 并发连接数
export CONCURRENT=3

# 请求数量
export REQUEST_COUNT=10

# 测试持续时间 (对于 -z 选项)
export DURATION="30s"

# ==================== 功能开关 ====================

# 启用/禁用缓存
export DISABLE_CACHE=true

# 页面资源收集级别: off | summary | full
export PAGE_RESOURCES="summary"

# 启用/禁用 TUI
export NO_TUI=true

# ==================== 输出配置 ====================

# 日志目录
export LOG_DIR="/tmp/spider_test_logs"

# 是否保留日志
export KEEP_LOGS=true

# 是否在测试后清理 Chrome 进程
export CLEANUP_CHROME=true

# ==================== 高级配置 ====================

# Chrome 启动参数
export CHROME_FLAGS="--no-sandbox --disable-dev-shm-usage"

# 页面超时时间 (秒)
export PAGE_TIMEOUT=30

# 是否使用 headless 模式
export HEADLESS=true

# ==================== 预检查 ====================

# 执行前检查的脚本
pre_check() {
    echo "预检查..."
    # 检查文件是否存在
    [ -f "$CHROME_BIN" ] || { echo "错误: 找不到 Chrome: $CHROME_BIN"; exit 1; }
    [ -x "$CHROME_BIN" ] || { echo "错误: Chrome 不可执行: $CHROME_BIN"; exit 1; }
    echo "✅ 预检查通过"
}

# ==================== 测试模板 ====================

# 基础 SMART 测试命令
smart_test_cmd() {
    echo "oha --page-loader spider --spider-mode smart \\
        --chrome-bin $CHROME_BIN \\
        $( [ "$DISABLE_CACHE" = "true" ] && echo "--spider-disable-cache" ) \\
        --page-resources $PAGE_RESOURCES \\
        $( [ "$NO_TUI" = "true" ] && echo "--no-tui" ) \\
        -c $CONCURRENT -n $REQUEST_COUNT \\
        $TEST_URL/"
}

# 资源收集测试命令
resource_test_cmd() {
    echo "oha --page-loader spider --spider-mode smart \\
        --chrome-bin $CHROME_BIN \\
        $( [ "$DISABLE_CACHE" = "true" ] && echo "--spider-disable-cache" ) \\
        --page-resources full \\
        -c $CONCURRENT -n $REQUEST_COUNT \\
        --no-tui \\
        $TEST_URL/"
}

# 性能测试命令
perf_test_cmd() {
    echo "oha --page-loader spider --spider-mode smart \\
        --chrome-bin $CHROME_BIN \\
        --spider-disable-cache \\
        --page-resources off \\
        -c $CONCURRENT -n $REQUEST_COUNT \\
        --no-tui \\
        $TEST_URL/"
}

# ==================== 使用示例 ====================

# 加载此配置文件后，您可以：
# 1. 修改上方的参数
# 2. 运行预检查: pre_check
# 3. 查看测试命令: smart_test_cmd
# 4. 执行测试: eval $(smart_test_cmd)

# 示例用法:
# source test_config.sh
# pre_check
# eval $(smart_test_cmd)

echo "测试配置已加载。如需修改，请编辑此文件或设置环境变量。"
echo "例如: export CONCURRENT=5"
