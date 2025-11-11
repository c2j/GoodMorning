//! Page recording functionality

use anyhow::Result;
use headless_chrome::Tab;
use std::collections::HashMap;
use std::time::Instant;
use super::types::{NetworkEvent, RequestData, ResponseData, ResourceType};

/// Network event collector for recording
pub struct NetworkCollector {
    events: Vec<NetworkEvent>,
    start_time: Instant,
    /// Track request start times for duration calculation
    request_starts: std::collections::HashMap<String, Instant>,
}

impl NetworkCollector {
    /// Create a new network collector
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            start_time: Instant::now(),
            request_starts: std::collections::HashMap::new(),
        }
    }

    /// Clear all collected events
    pub fn clear(&mut self) {
        self.events.clear();
        self.request_starts.clear();
        self.start_time = Instant::now();
    }

    /// Register a request event
    pub fn on_request(&mut self, request_id: &str, _url: &str, _method: &str) {
        // Record request start time
        self.request_starts.insert(request_id.to_string(), Instant::now());
    }

    /// Register a response event
    pub fn on_response(
        &mut self,
        request_id: &str,
        url: &str,
        method: &str,
        status: u16,
        headers: &HashMap<String, String>,
    ) {
        // Calculate actual duration
        let duration = if let Some(start_time) = self.request_starts.get(request_id) {
            start_time.elapsed()
        } else {
            // Fallback if we don't have start time
            std::time::Duration::from_millis(100)
        };

        let response_data = ResponseData {
            request_id: request_id.to_string(),
            status,
            headers: headers.clone(),
            body_size: None, // Can't easily get body size from headless_chrome
            duration,
        };

        // Create or update event
        self.events.push(NetworkEvent {
            request: RequestData {
                id: request_id.to_string(),
                url: url.to_string(),
                method: method.to_string(),
                resource_type: ResourceType::Other, // Will be determined by URL
                timestamp: self.start_time.elapsed(),
                headers: HashMap::new(), // TODO: Get request headers
            },
            response: Some(response_data),
        });
    }

    /// Get all collected events
    pub fn get_events(&self) -> &Vec<NetworkEvent> {
        &self.events
    }
}

/// Record a page load
pub async fn record_page_load(
    tab: &std::sync::Arc<Tab>,
    url: &str,
) -> Result<HashMap<String, serde_json::Value>> {
    let collector = NetworkCollector::new();

    // Set up network event listeners
    // Headless Chrome supports network monitoring via events

    // Navigate to page
    tab.navigate_to(url)?;

    // Wait for page to load
    tab.wait_until_navigated()?;

    // Get page metrics
    let remote_obj = tab.evaluate("({ title: document.title, url: document.URL })", false)?;

    // Convert to JSON value
    let metrics_json: serde_json::Value = match remote_obj.value {
        Some(v) => serde_json::from_value(v)?,
        None => serde_json::json!({}),
    };

    // Build result
    let mut result = HashMap::new();
    result.insert("events".to_string(), serde_json::to_value(collector.get_events())?);
    result.insert("metrics".to_string(), metrics_json);

    Ok(result)
}
