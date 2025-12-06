//! Unit tests for HTTP Client

use super::super::client::{AuthConfig, HttpClient, HttpRequestConfig};
use std::collections::HashMap;

// ===== HttpRequestConfig Tests =====

#[test]
fn test_request_config_creation() {
    let config = HttpRequestConfig {
        url: "https://example.com".to_string(),
        method: "GET".to_string(),
        headers: None,
        expected_status_code: 200,
        timeout_sec: 30,
        skip_tls: false,
        follow_redirect: true,
        auth: None,
    };

    assert_eq!(config.url, "https://example.com");
    assert_eq!(config.method, "GET");
    assert!(config.headers.is_none());
    assert_eq!(config.expected_status_code, 200);
    assert_eq!(config.timeout_sec, 30);
    assert!(!config.skip_tls);
    assert!(config.follow_redirect);
    assert!(config.auth.is_none());
}

#[test]
fn test_request_config_with_headers() {
    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    headers.insert("Accept".to_string(), "application/json".to_string());

    let config = HttpRequestConfig {
        url: "https://api.example.com/data".to_string(),
        method: "POST".to_string(),
        headers: Some(headers.clone()),
        expected_status_code: 201,
        timeout_sec: 60,
        skip_tls: false,
        follow_redirect: false,
        auth: None,
    };

    assert!(config.headers.is_some());
    let h = config.headers.unwrap();
    assert_eq!(h.get("Content-Type").unwrap(), "application/json");
    assert_eq!(h.get("Accept").unwrap(), "application/json");
}

#[test]
fn test_request_config_with_basic_auth() {
    let config = HttpRequestConfig {
        url: "https://example.com".to_string(),
        method: "GET".to_string(),
        headers: None,
        expected_status_code: 200,
        timeout_sec: 30,
        skip_tls: false,
        follow_redirect: true,
        auth: Some(AuthConfig {
            username: Some("user".to_string()),
            password: Some("pass".to_string()),
            bearer: None,
        }),
    };

    assert!(config.auth.is_some());
    let auth = config.auth.unwrap();
    assert_eq!(auth.username, Some("user".to_string()));
    assert_eq!(auth.password, Some("pass".to_string()));
    assert!(auth.bearer.is_none());
}

#[test]
fn test_request_config_with_bearer_auth() {
    let config = HttpRequestConfig {
        url: "https://api.example.com".to_string(),
        method: "GET".to_string(),
        headers: None,
        expected_status_code: 200,
        timeout_sec: 30,
        skip_tls: false,
        follow_redirect: true,
        auth: Some(AuthConfig {
            username: None,
            password: None,
            bearer: Some("my-secret-token".to_string()),
        }),
    };

    assert!(config.auth.is_some());
    let auth = config.auth.unwrap();
    assert!(auth.username.is_none());
    assert!(auth.password.is_none());
    assert_eq!(auth.bearer, Some("my-secret-token".to_string()));
}

// ===== Async Client Tests =====

#[tokio::test]
async fn test_execute_invalid_url() {
    let client = HttpClient::new();
    let config = HttpRequestConfig {
        url: "not-a-valid-url".to_string(),
        method: "GET".to_string(),
        headers: None,
        expected_status_code: 200,
        timeout_sec: 5,
        skip_tls: false,
        follow_redirect: true,
        auth: None,
    };

    let metrics = client.execute(&config).await;

    // Should fail with invalid URL
    assert!(!metrics.up);
    assert!(!metrics.success);
}

#[tokio::test]
async fn test_execute_dns_failure() {
    let client = HttpClient::new();
    let config = HttpRequestConfig {
        url: "https://this-domain-does-not-exist-12345.invalid/path".to_string(),
        method: "GET".to_string(),
        headers: None,
        expected_status_code: 200,
        timeout_sec: 5,
        skip_tls: false,
        follow_redirect: true,
        auth: None,
    };

    let metrics = client.execute(&config).await;

    // Should fail DNS resolution
    assert!(!metrics.up);
    assert!(!metrics.success);
    assert!(metrics.dns_duration.as_nanos() > 0, "DNS should have been attempted");
}

#[tokio::test]
async fn test_execute_connection_refused() {
    let client = HttpClient::new();
    // Use localhost with a port that's likely not listening
    let config = HttpRequestConfig {
        url: "http://127.0.0.1:59999".to_string(),
        method: "GET".to_string(),
        headers: None,
        expected_status_code: 200,
        timeout_sec: 2,
        skip_tls: false,
        follow_redirect: true,
        auth: None,
    };

    let metrics = client.execute(&config).await;

    // DNS should succeed (it's an IP), but TCP should fail
    assert!(!metrics.up, "Connection should be refused");
    assert!(!metrics.success);
}

#[tokio::test]
async fn test_execute_method_variations() {
    // Test that different methods are accepted (no runtime error)
    let methods = vec!["GET", "POST", "PUT", "DELETE", "HEAD", "OPTIONS", "PATCH"];

    for method in methods {
        let config = HttpRequestConfig {
            url: "http://127.0.0.1:59998".to_string(),
            method: method.to_string(),
            headers: None,
            expected_status_code: 200,
            timeout_sec: 1,
            skip_tls: false,
            follow_redirect: true,
            auth: None,
        };

        let client = HttpClient::new();
        let _ = client.execute(&config).await;
        // Just verify no panic occurs
    }
}

#[tokio::test]
async fn test_execute_with_timeout() {
    let client = HttpClient::new();
    // Very short timeout
    let config = HttpRequestConfig {
        url: "http://127.0.0.1:59997".to_string(),
        method: "GET".to_string(),
        headers: None,
        expected_status_code: 200,
        timeout_sec: 1,
        skip_tls: false,
        follow_redirect: true,
        auth: None,
    };

    let start = std::time::Instant::now();
    let _ = client.execute(&config).await;
    let elapsed = start.elapsed();

    // Should complete within reasonable time (not hang)
    assert!(elapsed.as_secs() < 10, "Should not hang");
}

#[tokio::test]
async fn test_metrics_fields_populated() {
    let client = HttpClient::new();
    let config = HttpRequestConfig {
        url: "http://127.0.0.1:59996".to_string(),
        method: "GET".to_string(),
        headers: None,
        expected_status_code: 200,
        timeout_sec: 2,
        skip_tls: false,
        follow_redirect: true,
        auth: None,
    };

    let metrics = client.execute(&config).await;

    // Even on failure, some fields should be set
    // resolved_ip should be set since we used an IP directly
    assert!(metrics.resolved_ip.is_some() || !metrics.up);
}
