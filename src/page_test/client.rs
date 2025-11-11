//! Headless Chrome client implementation

#[cfg(feature = "headless_chrome")]
use anyhow::Result;
use headless_chrome::Browser;
use std::sync::Arc;
use std::time::Duration;
use super::types::BrowserType;

/// Async retry helper function
#[cfg(feature = "headless_chrome")]
async fn retry_async<T, F, Fut>(
    max_retries: u32,
    delay: Duration,
    mut f: F,
) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut last_error = None;
    for attempt in 0..max_retries {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = Some(e);
                if attempt < max_retries - 1 {
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }
    Err(last_error.unwrap_or_else(|| anyhow::anyhow!("All retry attempts failed")))
}

/// Configuration for Headless Chrome client
#[cfg(feature = "headless_chrome")]
#[derive(Debug, Clone)]
pub struct PwConfig {
    pub page_timeout: Duration,
    pub connect_timeout: Duration,
    pub retry_count: u32,
    pub retry_delay: Duration,
}

#[cfg(feature = "headless_chrome")]
impl Default for PwConfig {
    fn default() -> Self {
        Self {
            page_timeout: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(5),
            retry_count: 3,
            retry_delay: Duration::from_millis(500),
        }
    }
}

/// Launch a browser instance
#[cfg(feature = "headless_chrome")]
async fn launch_browser(
    browser_type: BrowserType,
    headless: bool,
) -> Result<Browser> {
    let _ = browser_type; // Suppress unused variable warning
    let browser = headless_chrome::Browser::new(
        headless_chrome::LaunchOptionsBuilder::default()
            .headless(headless)
            .build()?,
    )?;
    Ok(browser)
}

/// Headless Chrome client for controlling browsers and pages
#[cfg(feature = "headless_chrome")]
pub struct PwClient {
    browser: Browser,
}

#[cfg(feature = "headless_chrome")]
impl PwClient {
    /// Create a new Headless Chrome client with default config
    pub async fn new(
        browser_type: BrowserType,
        headless: bool,
        page_timeout: Duration,
    ) -> Result<Self> {
        let config = PwConfig {
            page_timeout,
            ..Default::default()
        };
        Self::new_with_config(browser_type, headless, config).await
    }

    /// Create a new Headless Chrome client with custom config
    pub async fn new_with_config(
        browser_type: BrowserType,
        headless: bool,
        _config: PwConfig,
    ) -> Result<Self> {
        // Launch browser with retry
        let browser = retry_async(
            _config.retry_count,
            _config.retry_delay,
            || async { launch_browser(browser_type, headless).await }
        ).await?;

        Ok(Self {
            browser,
        })
    }

    /// Create a new page
    pub async fn new_page(&self) -> Result<Arc<headless_chrome::Tab>> {
        let tab = self.browser.new_tab()?;
        Ok(tab)
    }

    /// Close the browser and cleanup
    pub async fn close(self) -> Result<()> {
        // Browser will be closed when dropped
        Ok(())
    }
}

/// Placeholder client when headless_chrome feature is disabled
#[cfg(not(feature = "headless_chrome"))]
pub struct PwClient;

#[cfg(not(feature = "headless_chrome"))]
impl PwClient {
    /// Returns an error when Headless Chrome is not enabled
    pub async fn new(
        _browser_type: BrowserType,
        _headless: bool,
        _page_timeout: Duration,
    ) -> Result<Self> {
        Err(anyhow::anyhow!(
            "Headless Chrome support is not enabled. Enable the 'headless_chrome' feature."
        ))
    }

    /// Returns an error when Headless Chrome is not enabled
    pub async fn navigate_and_wait(&self, _url: &str) -> Result<()> {
        Err(anyhow::anyhow!(
            "Headless Chrome support is not enabled. Enable the 'headless_chrome' feature."
        ))
    }
}
