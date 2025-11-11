# Playwright 录制功能使用指南

## 修复说明
已解决以下问题：
1. ✅ 编译错误 - 所有编译问题已修复
2. ✅ 参数名冲突 - `--timeout` 和 `--pw-timeout` 的冲突已解决

## 使用方法

### 基本录制命令
```bash
# 录制页面并输出到控制台
./target/release/oha --pw-record https://example.com

# 录制并保存到文件
./target/release/oha --pw-record --pw-output scenario.json https://example.com
```

### 参数说明
- `--pw-record`: 启用录制模式
- `--pw-output <文件>`: 指定输出文件路径
- `--pw-browser <chromium|firefox|webkit>`: 选择浏览器类型 (默认: chromium)
- `--pw-headless`: 无头模式运行 (默认: true)
- `--pw-timeout <毫秒>`: 页面超时时间 (默认: 30000)

### 示例
```bash
# 使用 Firefox 录制
./target/release/oha --pw-record --pw-browser firefox https://example.com

# 保存到指定文件
./target/release/oha --pw-record --pw-output /tmp/scenario.json https://example.com

# 使用有头模式（可以看到浏览器窗口）
./target/release/oha --pw-record --pw-headless=false https://example.com
```

## 注意事项

### 系统要求
1. **安装 Playwright 浏览器**
   - 首次使用需要安装浏览器二进制文件
   - 运行时会自动尝试安装，或可手动安装：
     ```bash
     # 安装 Chromium
     playwright install chromium
     
     # 安装 Firefox
     playwright install firefox
     
     # 安装 WebKit
     playwright install webkit
     ```

2. **依赖库**
   - Linux 系统可能需要安装额外依赖：
     ```bash
     # Ubuntu/Debian
     sudo apt-get update
     sudo apt-get install -y libglib2.0-0 libnss3 libatk1.0-0 libatk-bridge2.0-0 libcups2 libdrm2 libxkbcommon0 libxcomposite1 libxdamage1 libxfixes3 libxrandr2 libgbm1 libasound2
     ```

### 当前功能限制
- ✅ 页面导航 - 可正常访问页面
- ✅ 基本指标收集 - 可获取页面标题和 URL
- ⚠️ 网络事件监控 - 当前 playwright-rust 版本不支持
- ⚠️ 详细性能指标 - 部分指标不可用

### 故障排除

**错误: "No such file or directory"**
- 原因: Playwright 浏览器未安装
- 解决: 运行 `playwright install chromium` 安装浏览器

**错误: "Failed to load page"**
- 原因: 网络问题或页面无法访问
- 解决: 检查 URL 是否正确，确保网络连接正常

**错误: "Failed to launch browser"**
- 原因: 缺少系统依赖或权限问题
- 解决: 安装系统依赖，或以无头模式运行

## 示例输出

录制成功时，会生成类似以下的 JSON 场景文件：
```json
{
  "version": "1.0",
  "created": "2025-11-11T00:00:00Z",
  "page": {
    "url": "https://example.com",
    "title": "Example Domain",
    "final_url": "https://example.com/"
  },
  "navigation": {
    "load_event_end": 0,
    "dom_content_loaded": 0,
    "first_paint": 0,
    "first_contentful_paint": 0
  },
  "resources": {
    "total_size": 0,
    "encoded_size": 0,
    "decoded_size": 0
  },
  "requests": []
}
```

## 重放模式 (TODO)
重放功能尚未完全实现，将在后续版本中完成。
