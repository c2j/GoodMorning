use spider::chromiumoxide::{Browser, BrowserConfig};
use std::sync::Arc;
use std::time::Duration;
use std::fs::File;
use std::io::Write;
use anyhow::Result;
use tokio::sync::Mutex;
use tokio::time::timeout as tokio_timeout;
use tokio::task::JoinHandle;
use spider::tokio_stream::StreamExt;
use spider::chromiumoxide::error::CdpError;
use chrono;

/// Manages a single Chrome instance with reusable page tabs
#[cfg(feature = "spider_smart")]
pub struct ChromeManager {
    browser: Arc<Mutex<Browser>>,
    /// The handler task that keeps the CDP connection alive
    handler_task: Option<JoinHandle<()>>,
    /// Track the number of active pages to prevent resource exhaustion
    active_pages: Arc<Mutex<usize>>,
    /// Maximum concurrent pages before we start throttling
    max_concurrent_pages: usize,
    /// Log file for debugging (to avoid interfering with TUI)
    log_file: Option<Arc<Mutex<File>>>,
    /// Whether cache is disabled
    disable_cache: bool,
}

#[cfg(feature = "spider_smart")]
impl ChromeManager {
    /// Launch a new Chrome browser instance
    ///
    /// # Arguments
    /// * `headless` - Whether to run Chrome in headless mode
    /// * `disable_cache` - Whether to disable Chrome cache
    /// * `chrome_bin` - Optional path to Chrome executable. If None, uses CHROME_BIN env var or system default
    pub async fn launch(headless: bool, disable_cache: bool, chrome_bin: Option<String>) -> Result<Self> {
        let mut browser_config = BrowserConfig::builder();

        // Set Chrome executable path if provided
        // Priority: 1. chrome_bin parameter, 2. CHROME_BIN env var, 3. system default
        let chrome_path = if let Some(path) = chrome_bin {
            // Convert relative path to absolute path
            let abs_path = if path.starts_with("./") || path.starts_with("../") {
                std::env::current_dir()
                    .ok()
                    .and_then(|cwd| cwd.join(&path).canonicalize().ok())
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or(path.clone())
            } else {
                path.clone()
            };
            browser_config = browser_config.chrome_executable(abs_path.clone());
            Some(abs_path)
        } else if let Ok(env_path) = std::env::var("CHROME_BIN") {
            browser_config = browser_config.chrome_executable(env_path.clone());
            Some(env_path)
        } else {
            None
        };

        // Note: with_head() actually shows the Chrome window
        // So we only call it when headless is FALSE
        if !headless {
            browser_config = browser_config.with_head();
        }

        // Add essential Chrome flags for Linux/headless environments
        // These flags are critical for Chrome to work properly in server/container environments
        browser_config = browser_config
            .arg("--no-sandbox")  // Required in many Linux environments
            .arg("--disable-dev-shm-usage")  // Overcome limited resource problems
            .arg("--disable-gpu")  // Disable GPU hardware acceleration
            .arg("--disable-software-rasterizer")  // Disable software rasterizer
            .arg("--allow-running-insecure-content");  // Allow mixed content (HTTP/HTTPS)

        // Set a unique user data directory to avoid conflicts
        let user_data_dir = format!("/tmp/chrome_{}", std::process::id());
        browser_config = browser_config.arg(format!("--user-data-dir={}", user_data_dir));

        // Add Chrome flags to disable cache if requested
        if disable_cache {
            browser_config = browser_config
                .arg("--disable-cache")
                .arg("--disable-application-cache")
                .arg("--disable-service-worker-cache");
        }

        let browser_config = browser_config
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to build browser config: {}", e))?;

        let (browser, mut handler) = Browser::launch(browser_config)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to launch Chrome: {}", e))?;

        // Create log file for debugging
        let log_file = match File::create("logs/chrome_manager.log") {
            Ok(f) => Some(Arc::new(Mutex::new(f))),
            Err(_) => None, // If we can't create log file, continue without logging
        };

        if let Some(ref log) = log_file {
            let mut log = log.lock().await;
            let chrome_info = chrome_path.as_ref()
                .map(|p| format!(" chrome_path: {}", p))
                .unwrap_or_else(|| " chrome_path: system default".to_string());
            let _ = log.write_all(format!("[{}] ChromeManager initialized (headless: {}, disable_cache: {},{})\n",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"), headless, disable_cache, chrome_info).as_bytes());
        }

        // Ensure logs directory exists
        if let Some(ref log) = log_file {
            let mut log = log.lock().await;
            let _ = log.write_all(b"\n=== ChromeManager Launched ===\n");
            let _ = log.write_all(format!("Chrome flags: --no-sandbox --disable-dev-shm-usage --disable-gpu --disable-software-rasterizer --allow-running-insecure-content --user-data-dir={}\n", user_data_dir).as_bytes());
            if disable_cache {
                let _ = log.write_all(b"Cache flags: --disable-cache --disable-application-cache --disable-service-worker-cache\n");
            }
            let _ = log.flush();
        }

        // Create a log file reference for the handler task
        let handler_log = log_file.clone();

        // Spawn a task to keep the handler alive - this is critical for CDP
        // Follow the same pattern as Spider library
        let handler_task = tokio::spawn(async move {
            if let Some(ref log) = handler_log {
                let mut log = log.lock().await;
                let _ = log.write_all(format!("[{}] Handler task started\n",
                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S")).as_bytes());
                let _ = log.flush();
            }

            let mut message_count = 0u64;
            let mut last_log_time = std::time::Instant::now();

            loop {
                match handler.next().await {
                    Some(Ok(_)) => {
                        // Successfully processed a message
                        message_count += 1;

                        // Log every 10 messages or every 5 seconds
                        if message_count % 10 == 0 || last_log_time.elapsed().as_secs() >= 5 {
                            if let Some(ref log) = handler_log {
                                let mut log = log.lock().await;
                                let _ = log.write_all(format!("[{}] Handler processed {} messages\n",
                                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S"), message_count).as_bytes());
                                let _ = log.flush();
                            }
                            last_log_time = std::time::Instant::now();
                        }
                    }
                    Some(Err(e)) => {
                        // Check if this is a non-critical Serde error (unknown CDP message)
                        let is_serde_error = matches!(e, CdpError::Serde(_));

                        // Only log non-Serde errors or log Serde errors at debug level
                        if !is_serde_error {
                            if let Some(ref log) = handler_log {
                                let mut log = log.lock().await;
                                let _ = log.write_all(format!("[{}] Handler error: {:?}\n",
                                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S"), e).as_bytes());
                                let _ = log.flush();
                            }
                        } else {
                            // Serde errors are usually just unknown CDP events from newer Chrome versions
                            // Count them but don't spam the log
                            // Only log the first few
                            static SERDE_ERROR_COUNT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
                            let count = SERDE_ERROR_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                            if count < 3 {
                                if let Some(ref log) = handler_log {
                                    let mut log = log.lock().await;
                                    let _ = log.write_all(format!("[{}] Handler: Ignoring unknown CDP message (Serde error #{}, future occurrences will be suppressed)\n",
                                        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"), count + 1).as_bytes());
                                    let _ = log.flush();
                                }
                            }
                        }

                        match e {
                            CdpError::Ws(_)
                            | CdpError::LaunchExit(_, _)
                            | CdpError::LaunchTimeout(_)
                            | CdpError::LaunchIo(_, _) => {
                                if let Some(ref log) = handler_log {
                                    let mut log = log.lock().await;
                                    let _ = log.write_all(format!("[{}] Handler task exiting due to critical error\n",
                                        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")).as_bytes());
                                    let _ = log.flush();
                                }
                                break;
                            }
                            _ => {
                                // Other errors (including Serde) are non-critical, continue processing
                                continue;
                            }
                        }
                    }
                    None => {
                        // Handler stream ended
                        if let Some(ref log) = handler_log {
                            let mut log = log.lock().await;
                            let _ = log.write_all(format!("[{}] Handler stream ended (None received)\n",
                                chrono::Local::now().format("%Y-%m-%d %H:%M:%S")).as_bytes());
                            let _ = log.flush();
                        }
                        break;
                    }
                }
            }

            if let Some(ref log) = handler_log {
                let mut log = log.lock().await;
                let _ = log.write_all(format!("[{}] Handler task ended (total messages: {})\n",
                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S"), message_count).as_bytes());
                let _ = log.flush();
            }
        });

        // Add a delay to allow Chrome's CDP server to fully initialize
        if let Some(ref log) = log_file {
            let mut log = log.lock().await;
            let _ = log.write_all(format!("[{}] Waiting 2s for Chrome CDP to initialize...\n",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S")).as_bytes());
            let _ = log.flush();
        }
        tokio::time::sleep(Duration::from_millis(2000)).await;

        if let Some(ref log) = log_file {
            let mut log = log.lock().await;
            let _ = log.write_all(format!("[{}] Chrome CDP initialization complete\n",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S")).as_bytes());
            let _ = log.flush();
        }

        // Try to get browser version to verify connection
        if let Some(ref log) = log_file {
            let mut log = log.lock().await;
            let _ = log.write_all(format!("[{}] Verifying Chrome connection...\n",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S")).as_bytes());
            let _ = log.flush();
        }

        // Test the connection by getting version
        match tokio::time::timeout(Duration::from_secs(5), browser.version()).await {
            Ok(Ok(version)) => {
                if let Some(ref log) = log_file {
                    let mut log = log.lock().await;
                    let _ = log.write_all(format!("[{}] Chrome connection verified: {}\n",
                        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"), version.product).as_bytes());
                    let _ = log.flush();
                }
            }
            Ok(Err(e)) => {
                if let Some(ref log) = log_file {
                    let mut log = log.lock().await;
                    let _ = log.write_all(format!("[{}] WARNING: Failed to get Chrome version: {}\n",
                        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"), e).as_bytes());
                    let _ = log.flush();
                }
            }
            Err(_) => {
                if let Some(ref log) = log_file {
                    let mut log = log.lock().await;
                    let _ = log.write_all(format!("[{}] WARNING: Timeout getting Chrome version\n",
                        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")).as_bytes());
                    let _ = log.flush();
                }
            }
        }

        Ok(ChromeManager {
            browser: Arc::new(Mutex::new(browser)),
            handler_task: Some(handler_task),
            active_pages: Arc::new(Mutex::new(0)),
            max_concurrent_pages: 3, // Reduced from 5 to 3 to reduce contention
            log_file,
            disable_cache,
        })
    }

    /// Connect to an existing Chrome instance via debugging URL
    pub async fn connect(url: &str) -> Result<Self> {
        // Use a default handler config - Spider uses its own handler config
        let (browser, _handler) = Browser::connect(url)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect to Chrome: {}", e))?;

        Ok(ChromeManager {
            browser: Arc::new(Mutex::new(browser)),
            handler_task: None, // No handler for connected browser - already managed
            active_pages: Arc::new(Mutex::new(0)),
            max_concurrent_pages: 3, // Reduced to reduce contention
            log_file: None, // Connecting to existing Chrome doesn't need log file
            disable_cache: false, // Cannot control cache of existing Chrome instance
        })
    }

    /// Write a log message to the log file (if available)
    async fn write_log(&self, message: &str) {
        if let Some(ref log_file) = self.log_file {
            // Use lock() instead of try_lock() to ensure all messages are written
            let mut log = log_file.lock().await;
            let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
            let _ = log.write_all(format!("[{}] {}\n", timestamp, message).as_bytes());
            let _ = log.flush();
        }
    }

    /// Check if cache is disabled
    pub fn is_cache_disabled(&self) -> bool {
        self.disable_cache
    }

    /// Create a new page tab on the existing Chrome instance with retry logic
    pub async fn create_page(&self) -> Result<spider::chromiumoxide::Page> {
        // First check if we're at the limit
        {
            let active = self.active_pages.lock().await;
            if *active >= self.max_concurrent_pages {
                self.write_log(&format!("At page limit ({}/{}), throttling...", *active, self.max_concurrent_pages)).await;
                // Wait a bit before trying again
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        }

        self.write_log("Creating new page (attempt 1/10)...").await;

        // Debug: Check browser connection status
        self.write_log(&format!("Browser Arc strong count: {}", Arc::strong_count(&self.browser))).await;

        // Try to create a page with retries in case of concurrent access
        // Increased from 5 to 10 attempts for better resilience
        for attempt in 0..10 {
            // Use longer timeout and add small delay before each attempt
            if attempt > 0 {
                let backoff_delay = if attempt <= 3 {
                    // Short backoff for first few attempts
                    Duration::from_millis(200 * attempt as u64)
                } else {
                    // Longer backoff for later attempts
                    Duration::from_millis(1000 + 500 * (attempt as u64 - 3))
                };
                self.write_log(&format!("Waiting {:?} before retry...", backoff_delay)).await;
                tokio::time::sleep(backoff_delay).await;
            }

            self.write_log(&format!("Attempt {}/10: Calling browser.new_page()...", attempt + 1)).await;
            let browser = self.browser.lock().await;
            match tokio_timeout(Duration::from_secs(45), browser.new_page("about:blank")).await {
                Ok(page_result) => match page_result {
                    Ok(page) => {
                        // Increment the active page count
                        {
                            let mut active = self.active_pages.lock().await;
                            *active += 1;
                            self.write_log(&format!("Page created successfully after {} attempts. Active pages: {}/{}",
                                attempt + 1, *active, self.max_concurrent_pages)).await;
                        }
                        return Ok(page);
                    }
                    Err(e) => {
                        let error_str = e.to_string();
                        self.write_log(&format!("Page creation attempt {} failed: {}", attempt + 1, error_str)).await;

                        // Special handling for "oneshot canceled" - be more patient
                        if error_str.contains("oneshot") {
                            self.write_log("Detected 'oneshot' error, using longer backoff...").await;
                            if attempt < 9 {
                                let delay = Duration::from_millis(1500);
                                self.write_log(&format!("Retrying in {:?}...", delay)).await;
                                tokio::time::sleep(delay).await;
                                continue;
                            }
                        }

                        if attempt < 9 {
                            // Wait before retrying with exponential backoff
                            let delay = if attempt < 3 {
                                Duration::from_millis(300 * (attempt as u64 + 1))
                            } else {
                                Duration::from_millis(800 + 400 * (attempt as u64 - 2))
                            };
                            self.write_log(&format!("Retrying in {:?}...", delay)).await;
                            tokio::time::sleep(delay).await;
                        } else {
                            let error_msg = format!("Failed to create page after 10 attempts: {}", e);
                            self.write_log(&format!("[ERROR] {}", error_msg)).await;
                            return Err(anyhow::anyhow!("{}", error_msg));
                        }
                    }
                },
                Err(_) => {
                    // Timeout
                    self.write_log(&format!("Page creation attempt {} timed out", attempt + 1)).await;
                    if attempt < 9 {
                        let delay = if attempt < 3 {
                            Duration::from_millis(500 * (attempt as u64 + 1))
                        } else {
                            Duration::from_millis(1000 + 500 * (attempt as u64 - 2))
                        };
                        self.write_log(&format!("Retrying in {:?}...", delay)).await;
                        tokio::time::sleep(delay).await;
                    } else {
                        let error_msg = "Failed to create page after 10 attempts: timeout".to_string();
                        self.write_log(&format!("[ERROR] {}", error_msg)).await;
                        return Err(anyhow::anyhow!("{}", error_msg));
                    }
                }
            }
        }

        unreachable!()
    }

    /// Close a specific page
    pub async fn close_page(&self, page: spider::chromiumoxide::Page) -> Result<()> {
        // Decrement the active page count
        {
            let mut active = self.active_pages.lock().await;
            if *active > 0 {
                *active -= 1;
            }
        }

        // Close the page
        page.close().await.map_err(|e| anyhow::anyhow!("Failed to close page: {}", e))
    }

    /// Get the browser reference (wrapped in Arc<Mutex<>>)
    pub fn browser(&self) -> &Arc<Mutex<Browser>> {
        &self.browser
    }

    /// Get the number of active pages
    pub async fn get_active_pages(&self) -> usize {
        let active = self.active_pages.lock().await;
        *active
    }

    /// Explicitly shutdown the Chrome browser and cleanup all resources
    /// This should be called when testing is complete to ensure no residual processes
    pub async fn shutdown(&self) -> Result<()> {
        self.write_log("Shutting down ChromeManager...").await;

        // First, close all pages
        let active_count = {
            let active = self.active_pages.lock().await;
            *active
        };

        if active_count > 0 {
            self.write_log(&format!("Warning: {} pages still active during shutdown", active_count)).await;
        }

        // Abort the handler task if it exists
        if let Some(ref handler) = self.handler_task {
            if !handler.is_finished() {
                self.write_log("Aborting handler task...").await;
                handler.abort();
            }
        }

        // Now we can get a mutable reference to the browser through the Mutex
        let mut browser = self.browser.lock().await;

        // Try to close the browser gracefully
        self.write_log("Requesting Chrome to close...").await;
        match browser.close().await {
            Ok(_) => {
                self.write_log("Chrome close request sent successfully").await;
            }
            Err(e) => {
                self.write_log(&format!("Warning: Failed to send close request: {}", e)).await;
            }
        }

        // Wait for the Chrome process to exit (with timeout)
        self.write_log("Waiting for Chrome process to exit...").await;
        match tokio::time::timeout(Duration::from_secs(5), browser.wait()).await {
            Ok(Ok(Some(status))) => {
                self.write_log(&format!("Chrome process exited with status: {:?}", status)).await;
            }
            Ok(Ok(None)) => {
                self.write_log("Chrome process exit status not available (not spawned by us)").await;
            }
            Ok(Err(e)) => {
                self.write_log(&format!("Error waiting for Chrome process: {}", e)).await;
            }
            Err(_) => {
                // Timeout - Chrome didn't exit gracefully, force kill it
                self.write_log("Timeout waiting for Chrome to exit, force killing...").await;
                match browser.kill().await {
                    Some(Ok(())) => {
                        self.write_log("Chrome process killed successfully").await;
                    }
                    Some(Err(e)) => {
                        self.write_log(&format!("Error killing Chrome process: {}", e)).await;
                    }
                    None => {
                        self.write_log("Chrome process not spawned by us, cannot kill").await;
                    }
                }
            }
        }

        self.write_log("ChromeManager shutdown complete").await;
        Ok(())
    }
}

#[cfg(feature = "spider_smart")]
impl Drop for ChromeManager {
    fn drop(&mut self) {
        // Abort handler task synchronously if possible
        if let Some(ref handler) = self.handler_task {
            if !handler.is_finished() {
                handler.abort();
            }
        }
        // Chrome will be automatically closed when Arc drops
        // Pages will be cleaned up by the browser's Drop implementation
    }
}
