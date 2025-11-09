//! Page recording functionality

use anyhow::Result;
use playwright::Page;
use serde_json::json;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use types::{NetworkEvent, RequestData, ResponseData, ResourceType};

/// Network event collector for recording
pub struct NetworkCollector {
    events: Vec<NetworkEvent>,
    start_time: Instant,
}

impl NetworkCollector {
    /// Create a new network collector
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            start_time: Instant::now(),
        }
    }

    /// Register a request event
    pub fn on_request(&mut self, request: &playwright::Request) {
        let request_data = RequestData {
            id: request.id().to_string(),
            url: request.url().to_string(),
            method: request.method().to_string(),
            resource_type: map_resource_type(request.resource_type()),
            timestamp: self.start_time.elapsed(),
            headers: parse_headers(request.headers()),
        };

        // Check if we already have this request
        for event in &mut self.events {
            if event.request.id == request_data.id {
                event.request = request_data;
                return;
            }
        }

        // Add new event
        self.events.push(NetworkEvent {
            request: request_data,
            response: None,
        });
    }

    /// Register a response event
    pub fn on_response(&mut self, response: &playwright::Response) {
        let response_data = ResponseData {
            request_id: response.request().id().to_string(),
            status: response.status(),
            headers: parse_headers(response.headers()),
            body_size: response.body().and_then(|b| {
                if b.is_empty() {
                    None
                } else {
                    Some(b.len() as u64)
                }
            }),
            // TODO: Calculate actual duration
            duration: Duration::from_millis(100),
        };

        // Update existing event
        for event in &mut self.events {
            if event.request.id == response_data.request_id {
                event.response = Some(response_data);
                return;
            }
        }

        // If we get a response without a request, create a new event
        let request = response.request();
        self.events.push(NetworkEvent {
            request: RequestData {
                id: request.id().to_string(),
                url: request.url().to_string(),
                method: request.method().to_string(),
                resource_type: map_resource_type(request.resource_type()),
                timestamp: self.start_time.elapsed(),
                headers: parse_headers(request.headers()),
            },
            response: Some(response_data),
        });
    }

    /// Get all collected events
    pub fn get_events(&self) -> &Vec<NetworkEvent> {
        &self.events
    }

    /// Clear all events
    pub fn clear(&mut self) {
        self.events.clear();
        self.start_time = Instant::now();
    }
}

/// Map Playwright resource type to our ResourceType enum
fn map_resource_type(resource_type: playwright::ResourceType) -> ResourceType {
    match resource_type {
        playwright::ResourceType::Document => ResourceType::Document,
        playwright::ResourceType::Stylesheet => ResourceType::Stylesheet,
        playwright::ResourceType::Script => ResourceType::Script,
        playwright::ResourceType::Image => ResourceType::Image,
        playwright::ResourceType::Font => ResourceType::Font,
        playwright::ResourceType::Media => ResourceType::Media,
        playwright::ResourceType::Xhr => ResourceType::Xhr,
        playwright::ResourceType::Fetch => ResourceType::Fetch,
        playwright::ResourceType::Other => ResourceType::Other,
    }
}

/// Parse headers from Playwright format
fn parse_headers(headers: &[playwright::Header]) -> HashMap<String, String> {
    headers
        .iter()
        .map(|h| (h.name.to_string(), h.value.to_string()))
        .collect()
}

/// Record a page load
pub async fn record_page_load(
    page: &Page,
    url: &str,
) -> Result<HashMap<String, serde_json::Value>> {
    let mut collector = NetworkCollector::new();

    // Set up network event listeners
    let page_for_events = page.clone();
    page.on_request(move |request| {
        let request = request.clone();
        collector.on_request(&request);
    });

    let page_for_response = page.clone();
    page.on_response(move |response| {
        let response = response.clone();
        collector.on_response(&response);
    });

    // Navigate to page
    let response = page.goto(url).await?;
    if let Some(response) = response {
        if !response.ok() {
            anyhow::bail!("Failed to load page: HTTP {}", response.status());
        }
    }

    // Wait for network to be idle
    page.wait_for_load_state(playwright::WaitUntil::NetworkIdle).await?;

    // Get page metrics
    let metrics = page.evaluate_expression(r#"
        ({
            title: document.title,
            url: document.URL,
            loadEventEnd: performance.timing.loadEventEnd - performance.timing.navigationStart,
            domContentLoaded: performance.timing.domContentLoadedEventEnd - performance.timing.navigationStart,
            firstPaint: performance.getEntriesByType('paint').find(entry => entry.name === 'first-paint')?.startTime || 0,
            firstContentfulPaint: performance.getEntriesByType('paint').find(entry => entry.name === 'first-contentful-paint')?.startTime || 0,
            transferSize: performance.getEntriesByType('resource').reduce((sum, r) => sum + r.transferSize, 0),
            encodedBodySize: performance.getEntriesByType('resource').reduce((sum, r) => sum + r.encodedBodySize, 0),
            decodedBodySize: performance.getEntriesByType('resource').reduce((sum, r) => sum + r.decodedBodySize, 0
        })
    "#).await?;

    // Build result
    let mut result = HashMap::new();
    result.insert("events".to_string(), serde_json::to_value(collector.get_events())?);
    result.insert("metrics".to_string(), metrics);

    Ok(result)
}
