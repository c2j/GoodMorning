# OHA Playwright 集成设计方案

## 1. 可行性评估

### 1.1 依赖库选择

**选择：c2j/playwright-rust (Playwright-1.50+)**

- **优势**：
  - 支持最新Playwright 1.50+ API
  - 积极维护，功能更完善
  - 可能包含官方版本未包含的优化
  - 更好的网络监控和拦截能力

- **评估结论**：✅ 可行，推荐使用

### 1.2 兼容性分析

| 组件 | 当前版本 | 兼容性 | 备注 |
|------|----------|--------|------|
| Rust Edition | 2024 | ✅ 兼容 | playwright-rust支持Rust 2021+ |
| Tokio | 1.38.1 | ✅ 兼容 | 需要检查是否需要调整 |
| tokio | features ["full"] | ✅ 兼容 | 支持所有必要特性 |
| async/await | stable | ✅ 兼容 | 完全支持 |

**总体兼容性：✅ 优秀**

### 1.3 功能覆盖度

| 需求 | playwright-rust支持 | 备注 |
|------|---------------------|------|
| 浏览器控制 | ✅ | launch、connect、attach |
| 网络拦截 | ✅ | route、request/response事件 |
| 网络监控 | ✅ | Network conditions、timings |
| 多浏览器 | ✅ | chromium、firefox、webkit |
| 页面导航 | ✅ | goto、wait_for_load_state |
| 元素操作 | ✅ | click、fill、select_option |
| 资源监控 | ✅ | 文档、资源、脚本、图片 |
| 截图/录制 | ✅ | 可选功能 |

**功能覆盖：✅ 100%**

## 2. 详细设计

### 2.1 依赖添加策略

#### 新增依赖 (Cargo.toml)
```toml
# Playwright支持
playwright = { git = "https://github.com/c2j/playwright-rust.git", branch = "Playwright-1.50+" }

# 可选：用于解析Playwright跟踪文件
playwright-api = { version = "0.1", optional = true }
```

#### 特性标志
```toml
[features]
default = ["rustls", "playwright"]  # 默认启用playwright
playwright = ["dep:playwright"]
# 其他特性...
```

### 2.2 模块架构

```
src/
├── page_test/                          # 页面测试新模块
│   ├── mod.rs                          # 主入口，聚合功能
│   ├── cli.rs                          # --pw-* CLI参数定义
│   ├── recorder.rs                     # 录制模式实现
│   ├── player.rs                       # 回放模式实现
│   ├── executor.rs                     # 场景执行引擎
│   ├── client.rs                       # Playwright客户端封装
│   ├── network.rs                      # 网络监控和拦截
│   ├── browser_pool.rs                 # 浏览器池管理
│   ├── stats.rs                        # 页面级统计
│   ├── printer.rs                      # 页面测试输出
│   └── types.rs                        # 类型定义
│
├── lib.rs                              # 扩展
│   ├── PageTestCommand                 # 新增命令类型
│   └── integrate_page_test()           # 集成函数
│
├── main.rs                             # 扩展
│   ├── parse_pw_args()                 # 解析--pw-*参数
│   └── dispatch_to_page_test()         # 分发到模块
│
├── printer.rs                          # 扩展
│   ├── print_page_json()               # JSON输出
│   ├── print_page_csv()                # CSV输出
│   └── print_page_tui()                # TUI显示
│
└── result_data.rs                      # 扩展
    ├── PageTestResult                  # 页面测试结果
    ├── PageMetrics                     # 页面级指标
    └── RequestDetail                   # 请求详情
```

### 2.3 数据流设计

#### 录制模式流程
```
CLI --pw-record --pw-output scenario.json →
解析参数 → 启动单浏览器 → 设置网络监控 →
执行页面操作 → 记录请求/响应 → 生成scenario.json
```

#### 回放模式流程
```
CLI --pw-replay --pw-scenario scenario.json →
加载场景 → 解析依赖图 → 预热浏览器池 →
为每个并发用户：
  → 申请浏览器实例
  → 创建页面
  → 执行场景（按依赖图）
  → 收集统计
  → 归还浏览器实例
→ 聚合结果 → 输出报告
```

### 2.4 关键API设计

#### Playwright客户端封装 (client.rs)

```rust
/// Playwright客户端主接口
pub struct PwClient {
    playwright: Playwright,
    browser: Browser,
    context: BrowserContext,
}

/// 支持的浏览器类型
#[derive(Debug, Clone)]
pub enum BrowserType {
    Chromium,
    Firefox,
    WebKit,
}

/// 网络事件回调
pub type NetworkCallback = Arc<dyn Fn(NetworkEvent) + Send + Sync>;

/// 初始化Playwright客户端
pub async fn create_client(
    browser_type: BrowserType,
    headless: bool,
    timeout: Duration,
) -> Result<PwClient> {
    let playwright = Playwright::init().await?;
    let browser = match browser_type {
        BrowserType::Chromium => playwright.chromium().launch().headless(headless).await?,
        BrowserType::Firefox => playwright.firefox().launch().headless(headless).await?,
        BrowserType::WebKit => playwright.webkit().launch().headless(headless).await?,
    };
    let context = browser.new_context().await?;
    Ok(PwClient { playwright, browser, context })
}

/// 导航到页面并等待加载
pub async fn navigate_and_wait(
    &self,
    url: &str,
    wait_until: WaitUntil,
    timeout: Duration,
) -> Result<PageLoadMetrics> {
    let page = self.context.new_page().await?;
    let start_time = Instant::now();

    // 设置网络监控
    let (tx, rx) = mpsc::channel(1000);
    page.on_request(move |req| {
        let _ = tx.send(NetworkEvent::Request(RequestData::from(req)));
    });
    page.on_response(move |res| {
        let _ = tx.send(NetworkEvent::Response(ResponseData::from(res)));
    });

    // 导航
    let response = page.goto(url).await?;
    page.wait_for_load_state(wait_until).await?;

    let load_time = start_time.elapsed();
    let load_metrics = PageLoadMetrics {
        url: url.to_string(),
        load_time,
        dom_content_loaded: page.evaluate_expression("performance.timing.domContentLoadedEventEnd - performance.timing.navigationStart").await?,
        ..Default::default()
    };

    Ok(load_metrics)
}
```

#### 场景执行引擎 (executor.rs)

```rust
/// 页面测试场景
#[derive(Debug, Clone)]
pub struct Scenario {
    pub page_url: String,
    pub actions: Vec<Action>,
    pub resources: ResourceFilter,
}

/// 支持的操作类型
#[derive(Debug, Clone)]
pub enum Action {
    Navigate { url: String },
    Wait { selector: Option<String>, timeout: Duration },
    Click { selector: String },
    Fill { selector: String, value: String },
    Select { selector: String, value: String },
    Xhr { url: String, method: String, body: Option<Value> },
    WaitForResponse { url_pattern: String },
}

/// 执行场景
pub struct ScenarioExecutor {
    client: PwClient,
    scenario: Scenario,
}

impl ScenarioExecutor {
    pub async fn execute(&self) -> Result<ScenarioResult> {
        let page = self.client.context.new_page().await?;

        let mut results = Vec::new();

        // 执行每个动作
        for action in &self.scenario.actions {
            let result = self.execute_action(&page, action).await?;
            results.push(result);
        }

        Ok(ScenarioResult { results })
    }

    async fn execute_action(&self, page: &Page, action: &Action) -> Result<ActionResult> {
        match action {
            Action::Navigate { url } => {
                let start = Instant::now();
                page.goto(url).await?;
                page.wait_for_load_state(WaitUntil::NetworkIdle).await?;
                Ok(ActionResult {
                    action: action.clone(),
                    duration: start.elapsed(),
                    status: ActionStatus::Success,
                })
            }
            Action::Wait { selector, timeout } => {
                let start = Instant::now();
                if let Some(sel) = selector {
                    page.wait_for_selector(sel).await?;
                } else {
                    tokio::time::sleep(*timeout).await;
                }
                Ok(ActionResult {
                    action: action.clone(),
                    duration: start.elapsed(),
                    status: ActionStatus::Success,
                })
            }
            // ... 其他操作
        }
    }
}
```

#### 浏览器池管理 (browser_pool.rs)

```rust
/// 浏览器池管理器
pub struct BrowserPool {
    pool: Arc<Mutex<Vec<PooledBrowser>>>,
    max_size: usize,
    client_factory: ClientFactory,
}

struct PooledBrowser {
    client: PwClient,
    last_used: Instant,
}

/// 从池中获取浏览器
pub async fn acquire(&self) -> Result<PooledBrowserHandle> {
    // 尝试从池中获取
    if let Some(browser) = self.pool.lock().await.pop() {
        return Ok(PooledBrowserHandle::new(browser, self));
    }

    // 池为空时创建新实例
    if self.current_size() < self.max_size {
        let client = self.client_factory.create().await?;
        Ok(PooledBrowserHandle::new_new(client, self))
    } else {
        // 等待池中释放
        self.wait_for_browser().await
    }
}

/// 预热池
pub async fn warm_up(&self, warm_size: usize) -> Result<()> {
    let mut handles = Vec::new();
    for _ in 0..warm_size {
        let handle = self.acquire().await?;
        handles.push(handle);
    }
    // 立即归还，不关闭
    for handle in handles {
        drop(handle);
    }
    Ok(())
}
```

### 2.5 CLI设计详解

#### 新增CLI参数 (cli.rs)

```rust
/// Playwright相关参数
#[derive(Parser, Debug)]
pub struct PwArgs {
    /// 启用录制模式
    #[arg(long = "pw-record")]
    pub record: bool,

    /// 启用回放模式
    #[arg(long = "pw-replay")]
    pub replay: bool,

    /// 场景文件输出路径（录制模式）
    #[arg(long = "pw-output", required_if("record", "true"))]
    pub output: Option<PathBuf>,

    /// 场景文件路径（回放/声明式模式）
    #[arg(long = "pw-scenario", required_if_any(["replay", "scenario_mode"]))]
    pub scenario: Option<PathBuf>,

    /// 浏览器类型
    #[arg(long = "pw-browser", default_value = "chromium")]
    pub browser: BrowserType,

    /// 无头模式
    #[arg(long = "pw-headless", default_value = "true")]
    pub headless: bool,

    /// 页面超时（毫秒）
    #[arg(long = "pw-timeout", default_value = "30000")]
    pub timeout: u64,

    /// 最大浏览器池大小
    #[arg(long = "pw-max-pool-size")]
    pub max_pool_size: Option<usize>,

    /// 预热池大小
    #[arg(long = "pw-pool-warmup")]
    pub pool_warmup: Option<usize>,

    /// 包含的资源模式
    #[arg(long = "pw-resource-include", value_delimiter = ',')]
    pub resource_include: Vec<String>,

    /// 排除的资源模式
    #[arg(long = "pw-resource-exclude", value_delimiter = ',')]
    pub resource_exclude: Vec<String>,

    /// 操作间隔（毫秒）
    #[arg(long = "pw-think-time", default_value = "0")]
    pub think_time: u64,

    /// 每用户并发页面数
    #[arg(long = "pw-concurrent-pages", default_value = "1")]
    pub concurrent_pages: usize,
}
```

#### 主命令集成 (main.rs)

```rust
#[tokio::main]
async fn main() -> Result<()> {
    // 解析CLI参数
    let args = Cli::parse();

    // 检查是否使用Playwright模式
    if args.pw_args.is_some() {
        let pw_args = args.pw_args.unwrap();

        if pw_args.record {
            // 录制模式
            return run_record_mode(pw_args).await;
        } else if pw_args.replay || pw_args.scenario.is_some() {
            // 回放或声明式模式
            return run_replay_mode(args, pw_args).await;
        }
    }

    // 原有HTTP模式
    // ...
}
```

### 2.6 统计指标设计

#### 页面级统计 (stats.rs)

```rust
/// 页面测试结果
#[derive(Debug, Clone)]
pub struct PageTestResult {
    /// 场景信息
    pub scenario: ScenarioInfo,

    /// 测试配置
    pub config: TestConfig,

    /// 页面级指标
    pub page_metrics: PageMetrics,

    /// 请求级统计
    pub request_stats: RequestStats,

    /// 每个用户的详细结果
    pub user_results: Vec<UserResult>,
}

/// 页面级指标
#[derive(Debug, Clone)]
pub struct PageMetrics {
    /// 首次加载时间 (TTFB)
    pub ttfb: Duration,

    /// 首次内容绘制 (FCP)
    pub first_contentful_paint: Duration,

    /// 最大内容绘制 (LCP)
    pub largest_contentful_paint: Duration,

    /// DOM内容加载时间
    pub dom_content_loaded: Duration,

    /// 页面加载完成时间
    pub page_load_complete: Duration,

    /// 资源总数
    pub resource_count: usize,

    /// 成功请求数
    pub success_count: usize,

    /// 失败请求数
    pub error_count: usize,

    /// 总传输大小
    pub total_size: u64,
}

/// 请求统计
#[derive(Debug, Clone)]
pub struct RequestStats {
    /// 请求总数
    pub count: u64,

    /// 成功率
    pub success_rate: f64,

    /// 平均响应时间
    pub avg_response_time: Duration,

    /// P50延迟
    pub p50: Duration,

    /// P90延迟
    pub p90: Duration,

    /// P99延迟
    pub p99: Duration,

    /// 每秒请求数
    pub rps: f64,
}

/// 用户结果
#[derive(Debug, Clone)]
pub struct UserResult {
    pub user_id: usize,
    pub page_results: Vec<PageResult>,
}

/// 单个页面结果
#[derive(Debug, Clone)]
pub struct PageResult {
    pub page_url: String,
    pub load_time: Duration,
    pub requests: Vec<RequestDetail>,
    pub errors: Vec<ErrorDetail>,
}

/// 请求详情
#[derive(Debug, Clone)]
pub struct RequestDetail {
    pub id: String,
    pub url: String,
    pub method: String,
    pub resource_type: ResourceType,
    pub start_time: Duration,
    pub duration: Duration,
    pub status: u16,
    pub size: u64,
    pub depends_on: Vec<String>,
}
```

### 2.7 输出格式设计

#### JSON输出格式 (printer.rs)

```rust
/// 打印页面测试结果为JSON
pub fn print_page_json(result: &PageTestResult, writer: &mut impl Write) -> Result<()> {
    let json = serde_json::to_string_pretty(result)?;
    writeln!(writer, "{}", json)?;
    Ok(())
}

/// JSON Schema for page test results
pub const PAGE_TEST_JSON_SCHEMA: &str = r#"
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "test_type": { "type": "string", "enum": ["page_test"] },
    "scenario": { "$ref": "#/definitions/ScenarioInfo" },
    "config": { "$ref": "#/definitions/TestConfig" },
    "page_metrics": { "$ref": "#/definitions/PageMetrics" },
    "request_stats": { "$ref": "#/definitions/RequestStats" },
    "user_results": {
      "type": "array",
      "items": { "$ref": "#/definitions/UserResult" }
    }
  }
}
"#;
```

#### CSV输出格式

```rust
/// 打印CSV格式
pub fn print_page_csv(result: &PageTestResult, writer: &mut impl Write) -> Result<()> {
    // 页面级CSV
    writeln!(writer, "Metric,Value")?;
    writeln!(writer, "Page URL,{}", result.page_metrics.page_url)?;
    writeln!(writer, "TTFB,{:?}", result.page_metrics.ttfb)?;
    writeln!(writer, "LCP,{:?}", result.page_metrics.largest_contentful_paint)?;
    writeln!(writer, "Total Requests,{}", result.page_metrics.resource_count)?;
    writeln!(writer, "Success Rate,{:.2}%", result.request_stats.success_rate * 100)?;

    // 请求级CSV
    writeln!(writer)?;
    writeln!(writer, "Request Details")?;
    writeln!(writer, "User,Page,URL,Method,Status,Duration(ms),Size(bytes)")?;

    for (user_idx, user_result) in result.user_results.iter().enumerate() {
        for page_result in &user_result.page_results {
            for request in &page_result.requests {
                writeln!(
                    writer,
                    "{},{},{},{},{},{},{}",
                    user_idx,
                    page_result.page_url,
                    request.url,
                    request.method,
                    request.status,
                    request.duration.as_millis(),
                    request.size
                )?;
            }
        }
    }

    Ok(())
}
```

## 3. 实现计划

### 阶段一：基础框架搭建

**预计工期：2-3周**

#### 任务1.1：依赖和环境准备
- [ ] 添加playwright依赖到Cargo.toml
- [ ] 配置特性标志
- [ ] 创建page_test模块结构
- [ ] 编写类型定义和基本接口

#### 任务1.2：CLI集成
- [ ] 实现PwArgs结构体
- [ ] 扩展主CLI解析逻辑
- [ ] 添加--pw-*参数支持
- [ ] 实现命令分发逻辑

#### 任务1.3：基础客户端
- [ ] 实现PwClient封装
- [ ] 添加浏览器启动/关闭逻辑
- [ ] 实现页面创建和导航
- [ ] 基础网络监控

#### 交付物：
- 基础框架可以编译
- 简单的页面导航测试通过
- CLI参数解析正确

### 阶段二：录制功能

**预计工期：2周**

#### 任务2.1：录制核心功能
- [ ] 实现网络事件监听
- [ ] 记录请求/响应详情
- [ ] 解析资源类型和大小
- [ ] 生成scenario.json

#### 任务2.2：用户交互记录
- [ ] 记录点击事件
- [ ] 记录输入事件
- [ ] 记录等待操作
- [ ] 记录XHR/Fetch请求

#### 任务2.3：场景验证
- [ ] 验证录制结果完整性
- [ ] 校验JSON格式
- [ ] 添加版本和元数据

#### 交付物：
- 可以录制完整页面加载过程
- scenario.json文件正确生成
- 包含所有资源请求和响应

### 阶段三：回放功能

**预计工期：3-4周**

#### 任务3.1：场景解析和验证
- [ ] 实现scenario.json解析
- [ ] 验证动作序列有效性
- [ ] 构建依赖图
- [ ] 声明式配置支持

#### 任务3.2：浏览器池管理
- [ ] 实现BrowserPool
- [ ] 添加预热机制
- [ ] 实现池回收逻辑
- [ ] 错误处理和恢复

#### 任务3.3：并发执行引擎
- [ ] 实现ScenarioExecutor
- [ ] 并发用户管理
- [ ] 执行结果收集
- [ ] 错误处理和重试

#### 交付物：
- 可以回放录制场景
- 支持多并发测试
- 稳定的执行引擎

### 阶段四：统计和输出

**预计工期：1-2周**

#### 任务4.1：统计计算
- [ ] 实现PageMetrics计算
- [ ] 实现RequestStats统计
- [ ] 延迟百分位数计算
- [ ] 成功/失败率统计

#### 任务4.2：输出格式
- [ ] 实现JSON输出
- [ ] 实现CSV输出
- [ ] 扩展TUI显示
- [ ] 数据库输出支持

#### 任务4.3：结果聚合
- [ ] 多用户结果聚合
- [ ] 页面级指标计算
- [ ] 请求级详细追踪
- [ ] 错误和异常报告

#### 交付物：
- 完整的统计报告
- 多种输出格式
- 用户友好的结果展示

### 阶段五：优化和测试

**预计工期：2-3周**

#### 任务5.1：性能优化
- [ ] 优化浏览器池管理
- [ ] 减少内存占用
- [ ] 提高并发效率
- [ ] 资源清理优化

#### 任务5.2：测试和验证
- [ ] 单元测试（>80%覆盖率）
- [ ] 集成测试
- [ ] 性能基准测试
- [ ] 端到端测试

#### 任务5.3：文档和示例
- [ ] API文档
- [ ] 使用指南
- [ ] 示例场景文件
- [ ] 最佳实践

#### 交付物：
- 生产就绪的功能
- 完整的测试覆盖
- 用户文档

## 4. 技术风险评估

### 4.1 高风险项

| 风险 | 影响 | 概率 | 缓解措施 |
|------|------|------|----------|
| 浏览器池内存泄漏 | 高 | 中 | 实现严格的资源清理，添加监控 |
| 浏览器启动时间过长 | 中 | 高 | 预热池，配置缓存，复用实例 |
| 并发限制导致性能瓶颈 | 中 | 中 | 优化池大小，限流控制 |
| Playwright版本兼容性 | 中 | 低 | 锁定版本，添加集成测试 |

### 4.2 中风险项

| 风险 | 影响 | 概率 | 缓解措施 |
|------|------|------|----------|
| 网络监控数据丢失 | 中 | 低 | 添加重试机制，缓冲队列 |
| 场景执行顺序问题 | 中 | 中 | 严格依赖图验证，单元测试 |
| 跨平台兼容性 | 中 | 低 | 多平台CI测试，容器化 |

### 4.3 低风险项

| 风险 | 影响 | 概率 | 缓解措施 |
|------|------|------|----------|
| JSON格式变更 | 低 | 低 | 版本控制，向后兼容 |
| CLI参数冲突 | 低 | 中 | 独立命名空间(--pw-*) |
| 文档不完整 | 低 | 中 | 分阶段文档，示例驱动 |

## 5. 性能基准

### 5.1 资源消耗

| 并发数 | 内存/实例 | CPU/实例 | 总内存 | 总CPU |
|--------|-----------|----------|--------|-------|
| 1 | 80MB | 10% | 80MB | 10% |
| 10 | 80MB | 10% | 800MB | 50-80% |
| 50 | 80MB | 10% | 4GB | 80-100% |
| 100 | 80MB | 10% | 8GB | 100% |

**建议最大并发：50-100（取决于硬件）**

### 5.2 执行效率

| 指标 | 预期值 | 备注 |
|------|--------|------|
| 启动时间 | 2-3秒 | 预热后可忽略 |
| 单页面执行时间 | 1-5秒 | 取决于页面复杂度 |
| 切换场景时间 | <500ms | 复用浏览器实例 |
| 统计输出时间 | <1秒 | 数据聚合时间 |

## 6. 测试策略

### 6.1 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scenario_parsing() {
        let scenario = read_scenario("test_data/basic.json").unwrap();
        assert_eq!(scenario.actions.len(), 3);
    }

    #[test]
    fn test_dependency_graph() {
        let requests = vec![
            Request { depends_on: vec![], id: "1" },
            Request { depends_on: vec!["1"], id: "2" },
        ];
        let graph = build_graph(&requests).unwrap();
        assert_eq!(graph.len(), 2);
    }
}
```

### 6.2 集成测试

```rust
#[tokio::test]
async fn test_basic_page_load() {
    let client = PwClient::create(BrowserType::Chromium, true, Duration::from_secs(30)).await.unwrap();
    let page = client.context.new_page().await.unwrap();

    let result = page.goto("https://example.com").await.unwrap();
    assert!(result.ok());
}
```

### 6.3 性能测试

```rust
#[tokio::test]
async fn test_concurrent_page_loads() {
    let pool = BrowserPool::new(10).await.unwrap();
    pool.warm_up(5).await.unwrap();

    let mut handles = Vec::new();
    for i in 0..20 {
        let handle = pool.acquire().await.unwrap();
        handles.push(async move {
            // 执行页面加载
        });
    }

    futures::future::join_all(handles).await;
    // 验证性能指标
}
```

## 7. 监控和调试

### 7.1 日志记录

```rust
#[derive(Debug)]
enum PwLogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl PwClient {
    pub fn set_log_level(&self, level: PwLogLevel) {
        // 配置Playwright日志
    }
}
```

### 7.2 性能追踪

```rust
struct PerformanceProfiler {
    spans: HashMap<String, Span>,
}

impl PerformanceProfiler {
    pub fn start_span(&mut self, name: &str) {
        self.spans.insert(name.to_string(), Span::start(name));
    }

    pub fn end_span(&mut self, name: &str) {
        if let Some(span) = self.spans.remove(name) {
            span.finish();
        }
    }
}
```

## 8. 总结

### 8.1 优势
- ✅ 基于成熟的Playwright库，功能强大
- ✅ 支持真实的浏览器行为测试
- ✅ 完整的网络监控和统计
- ✅ 灵活的录制-回放和声明式两种模式
- ✅ 与现有oha架构兼容性好

### 8.2 挑战
- ⚠️ 资源消耗较高（每个浏览器实例）
- ⚠️ 并发能力受限（受内存/CPU限制）
- ⚠️ 启动时间较长（需要预热优化）
- ⚠️ 实现复杂度高（需要池管理、状态同步）

### 8.3 建议
1. **分阶段实现**：先实现MVP（基础录制和回放），再添加高级功能
2. **资源控制**：严格限制并发数，提供配置选项
3. **用户体验**：提供详细的进度提示和错误信息
4. **文档完善**：提供丰富的示例和使用场景

---

**下一步行动**：
1. 获得开发确认
2. 搭建基础框架
3. 实现MVP功能
4. 迭代优化和测试

**预计总工期：10-14周**

**关键里程碑**：
- 第3周：基础框架完成
- 第6周：录制功能完成
- 第10周：回放功能完成
- 第13周：测试优化完成
- 第14周：发布