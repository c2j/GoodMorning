#[cfg(feature = "spider_page")]
use crate::client::{ClientError, RequestResult};
#[cfg(feature = "spider_page")]
use crate::pcg64si::Pcg64Si;
#[cfg(feature = "spider_smart")]
use crate::spider_chrome_manager::ChromeManager;
use std::io::Write;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[cfg(feature = "spider_page")]
use rand::{RngCore, SeedableRng};

#[cfg(feature = "spider_page")]
use url::Url;

/// Page load result containing aggregated metrics
#[cfg(feature = "spider_page")]
pub struct PageLoadResult {
    pub duration: Duration,
    pub bytes_total: usize,
    pub success: bool,
    pub resources: Vec<ResourceInfo>,
    /// DNS lookup time (if available from Chrome)
    pub dns_lookup: Option<Duration>,
    /// TCP connection time (if available from Chrome)
    pub tcp_connection: Option<Duration>,
    /// Time to first byte (if available from Chrome)
    pub first_byte: Option<Duration>,
    /// Time to DOM ready (if available from Chrome)
    pub dom_ready: Option<Duration>,
}

/// Information about a resource loaded by the page
#[cfg(feature = "spider_page")]
#[derive(Debug, Clone)]
pub struct ResourceInfo {
    pub url: String,
    pub size: usize,
    pub duration: Duration,
    pub status: u16,
    pub content_type: Option<String>,
}

/// Load a single page using Spider mode (HTTP or SMART)
#[cfg(feature = "spider_page")]
pub async fn load_page(
    url: &Url,
    timeout: Option<Duration>,
    mode: crate::SpiderMode,
    spider_opts: Option<&crate::SpiderOptions>,
) -> Result<PageLoadResult, ClientError> {
    load_page_with_chrome(url, timeout, mode, spider_opts, None).await
}

/// Load a single page using Spider mode with optional Chrome manager
#[cfg(all(feature = "spider_page", feature = "spider_smart"))]
pub async fn load_page_with_chrome(
    url: &Url,
    timeout: Option<Duration>,
    mode: crate::SpiderMode,
    spider_opts: Option<&crate::SpiderOptions>,
    chrome_manager: Option<&ChromeManager>,
) -> Result<PageLoadResult, ClientError> {
    match mode {
        crate::SpiderMode::Http => {
            load_page_http(url, timeout).await
        }
        crate::SpiderMode::Smart => {
            if let Some(chrome_mgr) = chrome_manager {
                load_page_smart_with_chrome(url, timeout, chrome_mgr, spider_opts).await
            } else {
                load_page_smart(url, timeout, spider_opts).await
            }
        }
    }
}

/// Load a single page using Spider mode with optional Chrome manager (no smart feature)
#[cfg(all(feature = "spider_page", not(feature = "spider_smart")))]
pub async fn load_page_with_chrome(
    url: &Url,
    timeout: Option<Duration>,
    mode: crate::SpiderMode,
    _spider_opts: Option<&crate::SpiderOptions>,
    _chrome_manager: Option<&()>,
) -> Result<PageLoadResult, ClientError> {
    match mode {
        crate::SpiderMode::Http => {
            load_page_http(url, timeout).await
        }
        crate::SpiderMode::Smart => {
            Err(ClientError::SpiderError(
                "SMART mode requires spider_smart feature".to_string()
            ))
        }
    }
}

/// Load a single page using Spider-HTTP mode
#[cfg(feature = "spider_page")]
async fn load_page_http(
    url: &Url,
    timeout: Option<Duration>,
) -> Result<PageLoadResult, ClientError> {
    let start = Instant::now();

    // First, fetch the main page HTML
    let (html, page_size, page_status) = fetch_main_page(url, timeout).await?;

    // Parse HTML to extract resource URLs
    let extracted_resources = extract_resources_from_html(&html, url);

    // Fetch and measure resources
    let mut bytes_total = page_size;
    let mut resources = Vec::new();

    for resource_url in extracted_resources {
        let resource_start = Instant::now();
        match fetch_resource(&resource_url, timeout).await {
            Ok((size, status)) => {
                let resource_duration = resource_start.elapsed();
                resources.push(ResourceInfo {
                    url: resource_url,
                    size,
                    duration: resource_duration,
                    status,
                    content_type: None,
                });
                bytes_total += size;
            }
            Err(_) => {
                // Resource fetch failed, but we still record it
                resources.push(ResourceInfo {
                    url: resource_url,
                    size: 0,
                    duration: resource_start.elapsed(),
                    status: 0, // Error status
                    content_type: None,
                });
            }
        }
    }

    let end = Instant::now();
    let duration = end - start;

    Ok(PageLoadResult {
        duration,
        bytes_total,
        success: page_status >= 200 && page_status < 300,
        resources,
        dns_lookup: None,
        tcp_connection: None,
        first_byte: None,
        dom_ready: None,
    })
}

/// Fetch the main page and return HTML, size, and status
#[cfg(feature = "spider_page")]
async fn fetch_main_page(url: &Url, timeout: Option<Duration>) -> Result<(String, usize, u16), ClientError> {
    let client = reqwest::Client::builder()
        .timeout(timeout.unwrap_or(Duration::from_secs(10)))
        .build()
        .map_err(|e| ClientError::SpiderError(e.to_string()))?;

    let response = client.get(url.as_str())
        .send()
        .await
        .map_err(|e| ClientError::SpiderError(e.to_string()))?;

    let status = response.status().as_u16();
    let bytes = response.bytes()
        .await
        .map_err(|e| ClientError::SpiderError(e.to_string()))?;

    let html = String::from_utf8(bytes.to_vec())
        .map_err(|e| ClientError::SpiderError(e.to_string()))?;

    Ok((html, bytes.len(), status))
}

/// Extract resource URLs from HTML
#[cfg(feature = "spider_page")]
fn extract_resources_from_html(html: &str, base_url: &Url) -> Vec<String> {
    use regex::Regex;

    let mut resources = Vec::new();

    // CSS files
    let css_regex = Regex::new(r#"href=["']([^"']*\.css[^"']*)["']"#).unwrap();
    for cap in css_regex.captures_iter(html) {
        if let Some(url) = cap.get(1) {
            let url_str = url.as_str();
            if let Ok(resolved) = base_url.join(url_str) {
                resources.push(resolved.to_string());
            }
        }
    }

    // JavaScript files
    let js_regex = Regex::new(r#"src=["']([^"']*\.js[^"']*)["']"#).unwrap();
    for cap in js_regex.captures_iter(html) {
        if let Some(url) = cap.get(1) {
            let url_str = url.as_str();
            if let Ok(resolved) = base_url.join(url_str) {
                resources.push(resolved.to_string());
            }
        }
    }

    // Images
    let img_regex = Regex::new(r#"src=["']([^"']*\.(?:png|jpg|jpeg|gif|webp|svg|ico)[^"']*)["']"#).unwrap();
    for cap in img_regex.captures_iter(html) {
        if let Some(url) = cap.get(1) {
            let url_str = url.as_str();
            if let Ok(resolved) = base_url.join(url_str) {
                resources.push(resolved.to_string());
            }
        }
    }

    // Fonts
    let font_regex = Regex::new(r#"href=["']([^"']*\.(?:woff|woff2|ttf|otf|eot)[^"']*)["']"#).unwrap();
    for cap in font_regex.captures_iter(html) {
        if let Some(url) = cap.get(1) {
            let url_str = url.as_str();
            if let Ok(resolved) = base_url.join(url_str) {
                resources.push(resolved.to_string());
            }
        }
    }

    resources
}

/// Load a single page using Spider-SMART mode (Chrome)
#[cfg(feature = "spider_smart")]
async fn load_page_smart(
    url: &Url,
    timeout: Option<Duration>,
    spider_opts: Option<&crate::SpiderOptions>,
) -> Result<PageLoadResult, ClientError> {
    use tokio::time::{Duration as TokioDuration, timeout as tokio_timeout};

    let start = Instant::now();

    // Create website with Chrome/SMART configuration
    let mut website = spider::website::Website::new(url.as_str())
        .with_limit(1)
        .with_depth(0)
        .with_subdomains(false)
        .with_caching(false)
        .with_chrome_connection(spider_opts.and_then(|o| o.chrome_url.clone()))
        .build()
        .map_err(|e| ClientError::SpiderError(e.to_string()))?;

    // Set timeout for crawling
    let timeout_duration = timeout.unwrap_or(TokioDuration::from_secs(30));

    // Crawl the page with timeout
    let crawl_result = tokio_timeout(timeout_duration, website.crawl()).await;

    match crawl_result {
        Ok(_) => {
            // Crawl succeeded (website.crawl() returns Result<(), E>)
            collect_smart_page_result(website, start, timeout).await
        }
        Err(_) => {
            // Timeout or crawl error
            Err(ClientError::SpiderError("Page load timeout".to_string()))
        }
    }
}

/// Load a page using Spider-SMART mode with a shared Chrome instance
#[cfg(feature = "spider_smart")]
async fn load_page_smart_with_chrome(
    url: &Url,
    timeout: Option<Duration>,
    chrome_manager: &ChromeManager,
    spider_opts: Option<&crate::SpiderOptions>,
) -> Result<PageLoadResult, ClientError> {
    use tokio::time::Duration as TokioDuration;

    let start = Instant::now();
    let timeout_duration = timeout.unwrap_or(TokioDuration::from_secs(30));

    // Create a new page tab on the existing Chrome instance
    let page = chrome_manager
        .create_page()
        .await
        .map_err(|e| ClientError::SpiderError(e.to_string()))?;

    // Check if we should collect resources based on page_resources setting
    let should_collect_resources = spider_opts
        .and_then(|o| Some(o.page_resources != crate::PageResources::Off))
        .unwrap_or(false);

    // Ensure page is closed when done
    let page_result = async {
        let status_code = 200u16; // Default status code
        let main_document_url = url.to_string();

        // Navigate to the URL with timeout
        let html_result = tokio::time::timeout(timeout_duration, async {
            page.goto(url.as_str())
                .await
                .map_err(|e| ClientError::SpiderError(format!("Failed to navigate: {}", e)))?;

            // Wait for page to load or timeout
            page.wait_for_navigation()
                .await
                .map_err(|e| ClientError::SpiderError(format!("Failed to wait for navigation: {}", e)))?;

            // Get the page content
            page.content()
                .await
                .map_err(|e| ClientError::SpiderError(format!("Failed to get content: {}", e)))
        }).await.map_err(|_| ClientError::SpiderError("Page load timeout".to_string()))?;

        let html = html_result?;
        let bytes = html.into_bytes();

        let end = Instant::now();
        let duration = end - start;

        // For now, we collect basic information only
        let resources = if should_collect_resources {
            vec![ResourceInfo {
                url: main_document_url,
                size: bytes.len(),
                duration,
                status: status_code,
                content_type: Some("text/html".to_string()),
            }]
        } else {
            Vec::new()
        };

        // Note: DNS, TCP, and DOM Ready times would require additional CDP domains
        Ok(PageLoadResult {
            duration,
            bytes_total: bytes.len(),
            success: status_code >= 200 && status_code < 300,
            resources,
            dns_lookup: None,
            tcp_connection: None,
            first_byte: None,
            dom_ready: None,
        })
    };

    let result = page_result.await;

    // Close the page regardless of success or failure
    if let Err(e) = chrome_manager.close_page(page).await {
        eprintln!("Warning: Failed to close page: {}", e);
    }

    result
}

/// Collect results from SMART mode crawl
#[cfg(feature = "spider_smart")]
async fn collect_smart_page_result(
    website: spider::website::Website,
    start: Instant,
    _timeout: Option<Duration>,
) -> Result<PageLoadResult, ClientError> {
    let mut bytes_total = 0;
    let resources = Vec::new();
    let mut success = true;

    let pages = website.get_pages();
    if let Some(page_vec) = pages {
        if let Some(first_page) = page_vec.first() {
            // Add main page size
            if let Some(content) = first_page.get_bytes() {
                bytes_total += content.len();
            }

            // Check if page loaded successfully
            if first_page.status_code.as_u16() < 200 || first_page.status_code.as_u16() >= 300 {
                success = false;
            }
        }
    }

    let end = Instant::now();
    let duration = end - start;

    Ok(PageLoadResult {
        duration,
        bytes_total,
        success,
        resources,
        dns_lookup: None,
        tcp_connection: None,
        first_byte: None,
        dom_ready: None,
    })
}

/// Fetch a single resource and return its size and status
#[cfg(feature = "spider_page")]
async fn fetch_resource(url: &str, timeout: Option<Duration>) -> Result<(usize, u16), ClientError> {
    let client = reqwest::Client::builder()
        .timeout(timeout.unwrap_or(Duration::from_secs(10)))
        .build()
        .map_err(|e| ClientError::SpiderError(e.to_string()))?;

    let response = client.get(url)
        .send()
        .await
        .map_err(|e| ClientError::SpiderError(e.to_string()))?;

    let status = response.status().as_u16();
    let bytes = response.bytes()
        .await
        .map_err(|e| ClientError::SpiderError(e.to_string()))?;

    Ok((bytes.len(), status))
}

/// Convert PageLoadResult to RequestResult for integration with existing result system
#[cfg(feature = "spider_page")]
pub fn page_result_to_request_result(
    page_result: PageLoadResult,
    rng_seed: u64,
) -> Result<RequestResult, ClientError> {
    let now = Instant::now();
    let start = now - page_result.duration;

    Ok(RequestResult {
        rng: Pcg64Si::seed_from_u64(rng_seed),
        start_latency_correction: None,
        start,
        connection_time: None, // Not applicable for page loads
        first_byte: None,      // Not available in Spider
        end: now,
        status: if page_result.success {
            hyper::StatusCode::OK
        } else {
            hyper::StatusCode::INTERNAL_SERVER_ERROR
        },
        len_bytes: page_result.bytes_total,
    })
}

/// Debug mode: perform a single page load and dump the results
#[cfg(feature = "spider_page")]
pub async fn spider_work_debug<W: Write>(
    output: &mut W,
    url: &str,
    spider_opts: Option<&crate::SpiderOptions>,
) -> Result<(), ClientError> {
    let url = Url::parse(url)
        .map_err(|e| ClientError::UrlParse(e))?;

    let timeout = spider_opts.and_then(|o| o.page_timeout.map(|d| d.into()));
    let mode = spider_opts.map(|o| o.spider_mode).unwrap_or(crate::SpiderMode::Http);

    let page_result = load_page(&url, timeout, mode, spider_opts).await?;

    writeln!(output, "Page Load Result:")?;
    writeln!(output, "  Duration: {:?}", page_result.duration)?;
    writeln!(output, "  Bytes Total: {}", page_result.bytes_total)?;
    writeln!(output, "  Success: {}", page_result.success)?;
    writeln!(output, "  Resources Count: {}", page_result.resources.len())?;

    for resource in &page_result.resources {
        writeln!(
            output,
            "    - {} ({} bytes)",
            resource.url, resource.size
        )?;
    }

    Ok(())
}

/// Main work function for Spider mode
#[cfg(all(feature = "spider_page", feature = "spider_smart"))]
pub async fn spider_work(
    url: &str,
    report_tx: kanal::Sender<Result<RequestResult, ClientError>>,
    n_tasks: usize,
    n_connections: usize,
    spider_opts: Option<&crate::SpiderOptions>,
) {
    spider_work_with_chrome(url, report_tx, n_tasks, n_connections, spider_opts, None).await
}

/// Main work function for Spider mode (no smart feature)
#[cfg(all(feature = "spider_page", not(feature = "spider_smart")))]
pub async fn spider_work(
    url: &str,
    report_tx: kanal::Sender<Result<RequestResult, ClientError>>,
    n_tasks: usize,
    n_connections: usize,
    spider_opts: Option<&crate::SpiderOptions>,
) {
    spider_work_with_chrome(url, report_tx, n_tasks, n_connections, spider_opts, None).await
}

/// Main work function for Spider mode with optional Chrome manager
#[cfg(all(feature = "spider_page", feature = "spider_smart"))]
pub async fn spider_work_with_chrome(
    url: &str,
    report_tx: kanal::Sender<Result<RequestResult, ClientError>>,
    n_tasks: usize,
    n_connections: usize,
    spider_opts: Option<&crate::SpiderOptions>,
    _shared_chrome_manager: Option<Arc<ChromeManager>>,
) {
    let url = match Url::parse(url) {
        Ok(url) => url,
        Err(e) => {
            let _ = report_tx.send(Err(ClientError::UrlParse(e)));
            return;
        }
    };

    let timeout = spider_opts.and_then(|o| o.page_timeout.map(|d| d.into()));
    let mode = spider_opts.map(|o| o.spider_mode).unwrap_or(crate::SpiderMode::Http);
    let headless = spider_opts.map(|o| o.spider_headless).unwrap_or(true);
    let disable_cache = spider_opts.map(|o| o.disable_cache).unwrap_or(false);
    let chrome_bin = spider_opts.and_then(|o| o.chrome_bin.clone());
    let page_resources = spider_opts.map(|o| o.page_resources).unwrap_or(crate::PageResources::Off);

    // For SMART mode: Create one Chrome instance per connection (n_connections Chrome processes)
    // For HTTP mode: No Chrome needed
    let chrome_managers = if mode == crate::SpiderMode::Smart {
        let mut managers = Vec::new();
        for i in 0..n_connections {
            match ChromeManager::launch(headless, disable_cache, chrome_bin.clone()).await {
                Ok(manager) => {
                    eprintln!("Chrome instance {}/{} launched successfully", i + 1, n_connections);
                    managers.push(Some(Arc::new(manager)));
                }
                Err(e) => {
                    eprintln!("Warning: Failed to launch Chrome instance {}/{}: {}", i + 1, n_connections, e);
                    managers.push(None);
                }
            }
        }
        managers
    } else {
        vec![None; n_connections]
    };

    // Each worker gets its own Chrome instance
    let futures = chrome_managers
        .into_iter()
        .enumerate()
        .map(|(worker_id, chrome_manager)| {
            let report_tx = report_tx.clone();
            let url = url.clone();
            let timeout = timeout;
            let mode = mode;
            let page_resources = page_resources;
            let headless = headless;
            let disable_cache = disable_cache;
            tokio::spawn(async move {
                let rng: Pcg64Si = rand::SeedableRng::from_os_rng();

                for _ in 0..(n_tasks / n_connections + 1) {
                    let page_result = if mode == crate::SpiderMode::Smart {
                        if let Some(ref chrome_mgr) = chrome_manager {
                            let spider_opts = crate::SpiderOptions {
                                page_loader: crate::PageLoader::Spider,
                                page_resources,
                                page_timeout: None,
                                spider_mode: mode,
                                spider_headless: headless,
                                disable_cache,
                                chrome_bin: None,
                                chrome_url: None,
                            };
                            load_page_with_chrome(&url, timeout, mode, Some(&spider_opts), Some(chrome_mgr)).await
                        } else {
                            // Chrome failed to launch for this worker, skip
                            eprintln!("Worker {} skipping request (no Chrome)", worker_id);
                            break;
                        }
                    } else {
                        load_page(&url, timeout, mode, None).await
                    };

                    match page_result {
                        Ok(page_result) => {
                            let seed = {
                                let mut temp_rng = rng.clone();
                                temp_rng.next_u64()
                            };
                            match page_result_to_request_result(page_result, seed) {
                                Ok(result) => {
                                    let _ = report_tx.send(Ok(result));
                                }
                                Err(err) => {
                                    let _ = report_tx.send(Err(err));
                                }
                            }
                        }
                        Err(err) => {
                            let _ = report_tx.send(Err(err));
                        }
                    }

                    if report_tx.is_closed() {
                        break;
                    }
                }

                // Shutdown this worker's Chrome instance
                if let Some(ref chrome_mgr) = chrome_manager {
                    let _ = chrome_mgr.shutdown().await;
                }
            })
        })
        .collect::<Vec<_>>();

    for f in futures {
        let _ = f.await;
    }
}

/// Main work function for Spider mode with time-based duration (SMART mode)
#[cfg(all(feature = "spider_page", feature = "spider_smart"))]
pub async fn spider_work_until_with_chrome(
    url: &str,
    report_tx: kanal::Sender<Result<RequestResult, ClientError>>,
    dead_line: std::time::Instant,
    n_connections: usize,
    spider_opts: Option<&crate::SpiderOptions>,
    _shared_chrome_manager: Option<Arc<ChromeManager>>,
) {
    let url = match Url::parse(url) {
        Ok(url) => url,
        Err(e) => {
            let _ = report_tx.send(Err(ClientError::UrlParse(e)));
            return;
        }
    };

    let timeout = spider_opts.and_then(|o| o.page_timeout.map(|d| d.into()));
    let mode = spider_opts.map(|o| o.spider_mode).unwrap_or(crate::SpiderMode::Http);
    let headless = spider_opts.map(|o| o.spider_headless).unwrap_or(true);
    let disable_cache = spider_opts.map(|o| o.disable_cache).unwrap_or(false);
    let chrome_bin = spider_opts.and_then(|o| o.chrome_bin.clone());
    let page_resources = spider_opts.map(|o| o.page_resources).unwrap_or(crate::PageResources::Off);

    // For SMART mode: Create one Chrome instance per connection (n_connections Chrome processes)
    // For HTTP mode: No Chrome needed
    let chrome_managers = if mode == crate::SpiderMode::Smart {
        let mut managers = Vec::new();
        for i in 0..n_connections {
            match ChromeManager::launch(headless, disable_cache, chrome_bin.clone()).await {
                Ok(manager) => {
                    eprintln!("Chrome instance {}/{} launched successfully", i + 1, n_connections);
                    managers.push(Some(Arc::new(manager)));
                }
                Err(e) => {
                    eprintln!("Warning: Failed to launch Chrome instance {}/{}: {}", i + 1, n_connections, e);
                    managers.push(None);
                }
            }
        }
        managers
    } else {
        vec![None; n_connections]
    };

    // Run until deadline - each worker gets its own Chrome instance
    let futures = chrome_managers
        .into_iter()
        .enumerate()
        .map(|(worker_id, chrome_manager)| {
            let report_tx = report_tx.clone();
            let url = url.clone();
            let timeout = timeout;
            let mode = mode;
            let page_resources = page_resources;
            let headless = headless;
            let disable_cache = disable_cache;
            tokio::spawn(async move {
                let rng: Pcg64Si = rand::SeedableRng::from_os_rng();

                // Keep sending requests until deadline
                while std::time::Instant::now() < dead_line {
                    let page_result = if mode == crate::SpiderMode::Smart {
                        if let Some(ref chrome_mgr) = chrome_manager {
                            let spider_opts = crate::SpiderOptions {
                                page_loader: crate::PageLoader::Spider,
                                page_resources,
                                page_timeout: None,
                                spider_mode: mode,
                                spider_headless: headless,
                                disable_cache,
                                chrome_bin: None,
                                chrome_url: None,
                            };
                            load_page_with_chrome(&url, timeout, mode, Some(&spider_opts), Some(chrome_mgr)).await
                        } else {
                            // Chrome failed to launch for this worker, skip
                            eprintln!("Worker {} skipping request (no Chrome)", worker_id);
                            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                            continue;
                        }
                    } else {
                        load_page(&url, timeout, mode, None).await
                    };

                    match page_result {
                        Ok(page_result) => {
                            let seed = {
                                let mut temp_rng = rng.clone();
                                temp_rng.next_u64()
                            };
                            match page_result_to_request_result(page_result, seed) {
                                Ok(result) => {
                                    let _ = report_tx.send(Ok(result));
                                }
                                Err(err) => {
                                    let _ = report_tx.send(Err(err));
                                }
                            }
                        }
                        Err(err) => {
                            let _ = report_tx.send(Err(err));
                        }
                    }

                    if report_tx.is_closed() || std::time::Instant::now() >= dead_line {
                        break;
                    }
                }

                // Shutdown this worker's Chrome instance
                if let Some(ref chrome_mgr) = chrome_manager {
                    let _ = chrome_mgr.shutdown().await;
                }
            })
        })
        .collect::<Vec<_>>();

    for f in futures {
        let _ = f.await;
    }
}

/// Main work function for Spider mode with optional Chrome manager (no smart feature)
#[cfg(all(feature = "spider_page", not(feature = "spider_smart")))]
pub async fn spider_work_with_chrome(
    url: &str,
    report_tx: kanal::Sender<Result<RequestResult, ClientError>>,
    n_tasks: usize,
    n_connections: usize,
    spider_opts: Option<&crate::SpiderOptions>,
    _chrome_manager: Option<Arc<()>>,
) {
    let url = match Url::parse(url) {
        Ok(url) => url,
        Err(e) => {
            let _ = report_tx.send(Err(ClientError::UrlParse(e)));
            return;
        }
    };

    let timeout = spider_opts.and_then(|o| o.page_timeout.map(|d| d.into()));
    let mode = spider_opts.map(|o| o.spider_mode).unwrap_or(crate::SpiderMode::Http);

    // For now, we use a simple concurrent approach
    // Spider-HTTP mode: each connection loads a page
    let futures = (0..n_connections)
        .map(|_| {
            let report_tx = report_tx.clone();
            let url = url.clone();
            let timeout = timeout;
            let mode = mode;
            tokio::spawn(async move {
                let rng: Pcg64Si = rand::SeedableRng::from_os_rng();

                for _ in 0..(n_tasks / n_connections + 1) {
                    let page_result = load_page(&url, timeout, mode, None).await;

                    match page_result {
                        Ok(page_result) => {
                            let seed = {
                                let mut temp_rng = rng.clone();
                                temp_rng.next_u64()
                            };
                            match page_result_to_request_result(page_result, seed) {
                                Ok(result) => {
                                    let _ = report_tx.send(Ok(result));
                                }
                                Err(err) => {
                                    let _ = report_tx.send(Err(err));
                                }
                            }
                        }
                        Err(err) => {
                            let _ = report_tx.send(Err(err));
                        }
                    }

                    if report_tx.is_closed() {
                        break;
                    }
                }
            })
        })
        .collect::<Vec<_>>();

    for f in futures {
        let _ = f.await;
    }
}
