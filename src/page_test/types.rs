//! Type definitions for Playwright-based page testing

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Supported browser types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserType {
    Chromium,
    Firefox,
    WebKit,
}

impl std::str::FromStr for BrowserType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "chromium" => Ok(BrowserType::Chromium),
            "firefox" => Ok(BrowserType::Firefox),
            "webkit" => Ok(BrowserType::WebKit),
            _ => Err(format!("Unknown browser type: {}", s)),
        }
    }
}

impl std::fmt::Display for BrowserType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BrowserType::Chromium => write!(f, "chromium"),
            BrowserType::Firefox => write!(f, "firefox"),
            BrowserType::WebKit => write!(f, "webkit"),
        }
    }
}

/// Test scenario information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioInfo {
    pub version: String,
    pub created: String,
    pub page_url: String,
    pub title: Option<String>,
    pub final_url: Option<String>,
}

/// Resource type classification
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    Document,
    Stylesheet,
    Script,
    Image,
    Font,
    Media,
    Xhr,
    Fetch,
    Other,
}

/// Network request data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestData {
    pub id: String,
    pub url: String,
    pub method: String,
    pub resource_type: ResourceType,
    pub timestamp: Duration,
    pub headers: std::collections::HashMap<String, String>,
}

/// Network response data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseData {
    pub request_id: String,
    pub status: u16,
    pub headers: std::collections::HashMap<String, String>,
    pub body_size: Option<u64>,
    pub duration: Duration,
}

/// Complete network event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkEvent {
    pub request: RequestData,
    pub response: Option<ResponseData>,
}

/// Test result structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub scenario: ScenarioInfo,
    pub page_metrics: PageMetrics,
    pub request_stats: RequestStats,
}

/// Page-level metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageMetrics {
    pub ttfb: Duration,
    pub dom_content_loaded: Duration,
    pub page_load_complete: Duration,
    pub resource_count: usize,
    pub success_count: usize,
    pub error_count: usize,
    pub total_size: u64,
}

/// Request-level statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestStats {
    pub count: u64,
    pub success_rate: f64,
    pub avg_response_time: Duration,
    pub p50: Duration,
    pub p90: Duration,
    pub p99: Duration,
}
