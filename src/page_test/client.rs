//! Playwright client implementation

#[cfg(feature = "playwright")]
use anyhow::Result;
use playwright::{Playwright, Browser, BrowserContext, Page};
use std::time::Duration;
use types::BrowserType;

/// Playwright client for controlling browsers and pages
#[cfg(feature = "playwright")]
pub struct PwClient {
    playwright: Playwright,
    browser: Browser,
    context: BrowserContext,
    _page: Option<Page>,
}

#[cfg(feature = "playwright")]
impl PwClient {
    /// Create a new Playwright client
    pub async fn new(
        browser_type: BrowserType,
        headless: bool,
        timeout: Duration,
    ) -> Result<Self> {
        // Initialize Playwright
        let playwright = Playwright::init().await?;

        // Launch browser
        let browser = match browser_type {
            BrowserType::Chromium => {
                playwright
                    .chromium()
                    .launcher()
                    .headless(headless)
                    .launch()
                    .await?
            }
            BrowserType::Firefox => {
                playwright
                    .firefox()
                    .launcher()
                    .headless(headless)
                    .launch()
                    .await?
            }
            BrowserType::WebKit => {
                playwright
                    .webkit()
                    .launcher()
                    .headless(headless)
                    .launch()
                    .await?
            }
        };

        // Create browser context
        let context = browser
            .new_context(Some(playwright.new_context_options().await?))
            .await?;

        Ok(Self {
            playwright,
            browser,
            context,
            _page: None,
        })
    }

    /// Create a new page
    pub async fn new_page(&self) -> Result<Page> {
        Ok(self.context.new_page().await?)
    }

    /// Close the browser and cleanup
    pub async fn close(self) -> Result<()> {
        drop(self.context);
        drop(self.browser);
        self.playwright.stop().await?;
        Ok(())
    }
}

/// Placeholder client when playwright feature is disabled
#[cfg(not(feature = "playwright"))]
pub struct PwClient;

#[cfg(not(feature = "playwright"))]
impl PwClient {
    /// Returns an error when Playwright is not enabled
    pub async fn new(
        _browser_type: BrowserType,
        _headless: bool,
        _timeout: Duration,
    ) -> Result<Self> {
        Err(anyhow::anyhow!(
            "Playwright support is not enabled. Enable the 'playwright' feature."
        ))
    }

    /// Returns an error when Playwright is not enabled
    pub async fn navigate_and_wait(&self, _url: &str) -> Result<()> {
        Err(anyhow::anyhow!(
            "Playwright support is not enabled. Enable the 'playwright' feature."
        ))
    }
}
