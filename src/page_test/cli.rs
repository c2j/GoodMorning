//! CLI arguments for Playwright-based page testing

use clap::Parser;
use std::path::PathBuf;
use types::BrowserType;

/// Playwright-related arguments
#[derive(Parser, Debug)]
#[group(multicall = true)]
pub struct PwArgs {
    /// Enable recording mode
    #[arg(long = "pw-record")]
    pub record: bool,

    /// Enable replay mode
    #[arg(long = "pw-replay")]
    pub replay: bool,

    /// Enable declarative mode (same as --pw-replay)
    #[arg(long = "pw-scenario-mode")]
    pub scenario_mode: bool,

    /// Scenario file output path (recording mode)
    #[arg(long = "pw-output", requires = "record")]
    pub output: Option<PathBuf>,

    /// Scenario file path (replay/declarative modes)
    #[arg(long = "pw-scenario", required_if_any(["replay", "scenario_mode"]))]
    pub scenario: Option<PathBuf>,

    /// Browser type to use
    #[arg(long = "pw-browser", default_value = "chromium")]
    pub browser: BrowserType,

    /// Run in headless mode
    #[arg(long = "pw-headless", default_value = "true")]
    pub headless: bool,

    /// Page timeout in milliseconds
    #[arg(long = "pw-timeout", default_value = "30000")]
    pub timeout: u64,

    /// Maximum browser pool size
    #[arg(long = "pw-max-pool-size")]
    pub max_pool_size: Option<usize>,

    /// Browser pool warmup size
    #[arg(long = "pw-pool-warmup")]
    pub pool_warmup: Option<usize>,

    /// Include resource patterns (comma-separated)
    #[arg(long = "pw-resource-include", value_delimiter = ',')]
    pub resource_include: Vec<String>,

    /// Exclude resource patterns (comma-separated)
    #[arg(long = "pw-resource-exclude", value_delimiter = ',')]
    pub resource_exclude: Vec<String>,

    /// Think time between actions in milliseconds
    #[arg(long = "pw-think-time", default_value = "0")]
    pub think_time: u64,

    /// Number of concurrent pages per user
    #[arg(long = "pw-concurrent-pages", default_value = "1")]
    pub concurrent_pages: usize,
}
