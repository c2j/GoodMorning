//! Unit tests for Headless Chrome integration

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_network_collector_new() {
        let collector = NetworkCollector::new();
        assert!(collector.get_events().is_empty());
    }

    #[test]
    fn test_network_collector_clear() {
        let mut collector = NetworkCollector::new();
        // Simulate adding some events
        collector.clear();
        assert!(collector.get_events().is_empty());
    }

    #[test]
    fn test_pw_config_default() {
        let config = PwConfig::default();
        assert_eq!(config.page_timeout, Duration::from_secs(30));
        assert_eq!(config.connect_timeout, Duration::from_secs(5));
        assert_eq!(config.retry_count, 3);
        assert_eq!(config.retry_delay, Duration::from_millis(500));
    }

    #[test]
    fn test_pw_config_custom() {
        let config = PwConfig {
            page_timeout: Duration::from_secs(60),
            connect_timeout: Duration::from_secs(10),
            retry_count: 5,
            retry_delay: Duration::from_secs(1),
        };
        assert_eq!(config.page_timeout, Duration::from_secs(60));
        assert_eq!(config.retry_count, 5);
    }

    #[test]
    fn test_validate_scenario_valid() {
        let scenario = serde_json::json!({
            "version": "1.0",
            "page": {
                "url": "https://example.com"
            },
            "requests": [
                {
                    "url": "https://example.com",
                    "method": "GET"
                }
            ]
        });

        assert!(validate_scenario(&scenario).is_ok());
    }

    #[test]
    fn test_validate_scenario_missing_version() {
        let scenario = serde_json::json!({
            "page": {
                "url": "https://example.com"
            },
            "requests": []
        });

        assert!(validate_scenario(&scenario).is_err());
    }

    #[test]
    fn test_validate_scenario_missing_page() {
        let scenario = serde_json::json!({
            "version": "1.0",
            "requests": []
        });

        assert!(validate_scenario(&scenario).is_err());
    }

    #[test]
    fn test_validate_scenario_missing_page_url() {
        let scenario = serde_json::json!({
            "version": "1.0",
            "page": {},
            "requests": []
        });

        assert!(validate_scenario(&scenario).is_err());
    }

    #[test]
    fn test_validate_scenario_missing_requests() {
        let scenario = serde_json::json!({
            "version": "1.0",
            "page": {
                "url": "https://example.com"
            }
        });

        assert!(validate_scenario(&scenario).is_err());
    }

    #[test]
    fn test_validate_scenario_requests_not_array() {
        let scenario = serde_json::json!({
            "version": "1.0",
            "page": {
                "url": "https://example.com"
            },
            "requests": "not an array"
        });

        assert!(validate_scenario(&scenario).is_err());
    }

    #[test]
    fn test_validate_scenario_request_missing_url() {
        let scenario = serde_json::json!({
            "version": "1.0",
            "page": {
                "url": "https://example.com"
            },
            "requests": [
                {
                    "method": "GET"
                }
            ]
        });

        assert!(validate_scenario(&scenario).is_err());
    }

    #[test]
    fn test_validate_scenario_request_missing_method() {
        let scenario = serde_json::json!({
            "version": "1.0",
            "page": {
                "url": "https://example.com"
            },
            "requests": [
                {
                    "url": "https://example.com"
                }
            ]
        });

        assert!(validate_scenario(&scenario).is_err());
    }

    #[test]
    fn test_validate_declarative_config_valid() {
        let config = serde_json::json!({
            "page": {
                "url": "https://example.com"
            },
            "actions": [
                {
                    "type": "navigate",
                    "url": "https://example.com"
                }
            ]
        });

        assert!(validate_declarative_config(&config).is_ok());
    }

    #[test]
    fn test_validate_declarative_config_missing_page() {
        let config = serde_json::json!({
            "actions": []
        });

        assert!(validate_declarative_config(&config).is_err());
    }

    #[test]
    fn test_validate_declarative_config_missing_actions() {
        let config = serde_json::json!({
            "page": {
                "url": "https://example.com"
            }
        });

        assert!(validate_declarative_config(&config).is_err());
    }

    #[test]
    fn test_validate_declarative_config_action_missing_type() {
        let config = serde_json::json!({
            "page": {
                "url": "https://example.com"
            },
            "actions": [
                {
                    "url": "https://example.com"
                }
            ]
        });

        assert!(validate_declarative_config(&config).is_err());
    }

    #[test]
    fn test_validate_declarative_config_actions_not_array() {
        let config = serde_json::json!({
            "page": {
                "url": "https://example.com"
            },
            "actions": "not an array"
        });

        assert!(validate_declarative_config(&config).is_err());
    }
}
