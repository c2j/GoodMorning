# OHA Playwright 录制功能使用指南

## 概述

OHA 现在支持使用 Playwright 进行真实的页面性能测试录制功能。录制模式允许你导航到一个页面，自动记录所有网络请求和响应，并生成一个 scenario.json 文件，该文件可用于后续的回放测试。

## 功能特性

- ✅ 自动记录页面加载的所有网络请求（HTML、CSS、JS、图片、XHR 等）
- ✅ 记录页面性能指标（TTFB、LCP、FCP、DOM 内容加载时间等）
- ✅ 生成结构化的 scenario.json 文件
- ✅ 支持多种浏览器（Chromium、Firefox、WebKit）
- ✅ 支持无头和有头模式

## 使用方法

### 基本录制命令

```bash
# 录制一个页面并保存到文件
oha --pw-record --pw-output scenario.json -c 1 -n 1 https://example.com

# 录制并输出到 stdout
oha --pw-record https://example.com

# 使用指定浏览器
oha --pw-record --pw-output scenario.json --pw-browser firefox https://example.com
```

### 完整示例

```bash
# 录制示例网站
oha --pw-record \
    --pw-output my-scenario.json \
    --pw-browser chromium \
    --pw-headless true \
    --pw-timeout 30000 \
    -c 1 \
    -n 1 \
    https://example.com
```

## 生成的 Scenario JSON

录制的 scenario.json 文件包含以下信息：

```json
{
  "version": "1.0",
  "created": "2025-11-09T10:00:00Z",
  "page": {
    "url": "https://example.com",
    "title": "Example Page",
    "final_url": "https://example.com"
  },
  "navigation": {
    "load_event_end": 1234,
    "dom_content_loaded": 567,
    "first_paint": 234,
    "first_contentful_paint": 345
  },
  "resources": {
    "total_size": 567890,
    "encoded_size": 234567,
    "decoded_size": 345678
  },
  "requests": [
    {
      "id": "req_1",
      "url": "https://example.com/",
      "method": "GET",
      "resource_type": "document",
      "timestamp": 0,
      "headers": {
        "accept": "text/html"
      },
      "response": {
        "status": 200,
        "headers": {
          "content-type": "text/html"
        },
        "body_size": 12345,
        "duration": 234
      }
    }
  ]
}
```

## 字段说明

### Page 信息
- `url`: 录制时的原始 URL
- `title`: 页面标题
- `final_url`: 加载完成后的最终 URL（可能有重定向）

### Navigation 指标
- `load_event_end`: 页面 load 事件完成时间（ms）
- `dom_content_loaded`: DOM 内容加载完成时间（ms）
- `first_paint`: 首次绘制时间（ms）
- `first_contentful_paint`: 首次内容绘制时间（ms）

### Resources 汇总
- `total_size`: 传输的总大小（bytes）
- `encoded_size`: 压缩后大小（bytes）
- `decoded_size`: 解压后大小（bytes）

### 请求详情
每个请求包含：
- `id`: 请求唯一标识符
- `url`: 请求 URL
- `method`: HTTP 方法（GET、POST 等）
- `resource_type`: 资源类型（document、stylesheet、script、image、xhr 等）
- `timestamp`: 相对于页面开始加载的时间（ms）
- `headers`: 请求头
- `response`: 响应信息
  - `status`: HTTP 状态码
  - `headers`: 响应头
  - `body_size`: 响应体大小（bytes）
  - `duration`: 请求耗时（ms）

## CLI 参数详解

### 必需参数

- `--pw-record`: 启用录制模式
- `URL`: 要录制的页面 URL

### 可选参数

| 参数 | 说明 | 默认值 |
|------|------|--------|
| `--pw-output <path>` | scenario.json 输出路径 | stdout |
| `--pw-browser <type>` | 浏览器类型（chromium/firefox/webkit） | chromium |
| `--pw-headless` | 无头模式 | true |
| `--pw-timeout <ms>` | 页面超时时间（毫秒） | 30000 |

## 性能测试场景

### 场景 1: 基本页面性能测试

```bash
oha --pw-record --pw-output homepage.json https://your-site.com
```

### 场景 2: API 页面测试

```bash
oha --pw-record --pw-output api-page.json https://api.example.com/endpoint
```

### 场景 3: 单页应用（SPA）测试

```bash
oha --pw-record --pw-output spa.json -c 1 -n 1 https://your-spa.com
```

## 注意事项

1. **浏览器要求**: 确保已安装 Playwright 浏览器
   ```bash
   playwright install chromium
   playwright install firefox
   playwright install webkit
   ```

2. **网络稳定性**: 录制期间请确保网络连接稳定

3. **缓存影响**: 首次访问可能会有更多请求（缓存未命中），重复访问可能会有不同的结果

4. **动态内容**: 如果页面包含随机或时间敏感的内容，每次录制可能会不同

5. **资源限制**: 在高并发情况下，浏览器可能会消耗较多内存

## 下一步

录制生成的 scenario.json 文件可以用于：

1. **回放测试**: 使用 `oha --pw-replay` 进行多并发回放
2. **性能分析**: 分析页面加载性能
3. **网络优化**: 识别可以优化的资源
4. **CI/CD 集成**: 在持续集成中监控性能

## 相关命令

```bash
# 查看所有 Playwright 相关参数
oha --help | grep pw-

# 列出浏览器类型
oha --pw-browser chromium --help

# 录制模式
oha --pw-record

# 回放模式（后续阶段）
oha --pw-replay
```

## 示例输出

```bash
$ oha --pw-record --pw-output example.json https://example.com

Scenario saved to: example.json
```

## 故障排除

### 问题 1: "browser not found"
**解决**: 安装 Playwright 浏览器
```bash
playwright install chromium
```

### 问题 2: "Failed to load page"
**解决**: 检查 URL 是否可访问，或增加 `--pw-timeout`

### 问题 3: "Network timeout"
**解决**: 增加超时时间
```bash
oha --pw-record --pw-timeout 60000 <url>
```

---

**版本**: OHA v1.11.0+
**最后更新**: 2025-11-09
