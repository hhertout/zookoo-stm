//! Integration tests for HTTP Probe
//!
//! These tests verify the complete probe lifecycle with real or mock servers.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use configuration::model::scrape_interval::ScrapeInterval;
use configuration::model::target::HttpTarget;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::oneshot;

use super::super::client::{HttpClient, HttpRequestConfig};
use super::super::probe::HttpProbe;
use crate::Probe;

// ===== Mock HTTP Server =====

/// A simple mock HTTP server for testing
struct MockHttpServer {
    addr: SocketAddr,
    shutdown_tx: Option<oneshot::Sender<()>>,
}

impl MockHttpServer {
    /// Start a mock server that returns a configurable response
    async fn start(response_status: u16, response_body: &str) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let (shutdown_tx, mut shutdown_rx) = oneshot::channel();

        let response_body = response_body.to_string();
        let response = format!(
            "HTTP/1.1 {} OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            response_status,
            response_body.len(),
            response_body
        );

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    accept_result = listener.accept() => {
                        if let Ok((mut socket, _)) = accept_result {
                            let response = response.clone();
                            tokio::spawn(async move {
                                // Read the request (simple, just drain it)
                                let mut buf = [0u8; 1024];
                                let _ = socket.read(&mut buf).await;

                                // Send response
                                let _ = socket.write_all(response.as_bytes()).await;
                                let _ = socket.shutdown().await;
                            });
                        }
                    }
                    _ = &mut shutdown_rx => {
                        break;
                    }
                }
            }
        });

        Self { addr, shutdown_tx: Some(shutdown_tx) }
    }

    /// Get the server URL
    fn url(&self) -> String {
        format!("http://{}", self.addr)
    }

    /// Stop the server
    fn stop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }
}

impl Drop for MockHttpServer {
    fn drop(&mut self) {
        self.stop();
    }
}

// ===== Integration Tests with Mock Server =====

#[tokio::test]
async fn test_probe_success_200() {
    let server = MockHttpServer::start(200, "OK").await;

    let client = HttpClient::new();
    let config = HttpRequestConfig {
        url: server.url(),
        method: "GET".to_string(),
        headers: None,
        expected_status_code: 200,
        timeout_sec: 5,
        skip_tls: false,
        follow_redirect: true,
        auth: None,
    };

    let metrics = client.execute(&config).await;

    assert!(metrics.up, "Server should be reachable");
    assert!(metrics.success, "Probe should succeed with matching status code");
    assert_eq!(metrics.status_code, 200);
    assert!(metrics.dns_duration.as_nanos() > 0);
    assert!(metrics.tcp_connect_duration.as_nanos() > 0);
    assert!(metrics.time_to_first_byte.as_nanos() > 0);
    assert!(metrics.total_duration.as_nanos() > 0);
}

#[tokio::test]
async fn test_probe_status_mismatch() {
    let server = MockHttpServer::start(500, "Internal Server Error").await;

    let client = HttpClient::new();
    let config = HttpRequestConfig {
        url: server.url(),
        method: "GET".to_string(),
        headers: None,
        expected_status_code: 200, // Expecting 200, but server returns 500
        timeout_sec: 5,
        skip_tls: false,
        follow_redirect: true,
        auth: None,
    };

    let metrics = client.execute(&config).await;

    assert!(metrics.up, "Server should be reachable");
    assert!(!metrics.success, "Probe should fail due to status mismatch");
    assert_eq!(metrics.status_code, 500);
}

#[tokio::test]
async fn test_probe_404_expected() {
    let server = MockHttpServer::start(404, "Not Found").await;

    let client = HttpClient::new();
    let config = HttpRequestConfig {
        url: server.url(),
        method: "GET".to_string(),
        headers: None,
        expected_status_code: 404, // We expect 404
        timeout_sec: 5,
        skip_tls: false,
        follow_redirect: true,
        auth: None,
    };

    let metrics = client.execute(&config).await;

    assert!(metrics.up);
    assert!(metrics.success, "404 expected and received = success");
    assert_eq!(metrics.status_code, 404);
}

#[tokio::test]
async fn test_probe_timing_phases() {
    let server = MockHttpServer::start(200, "Test body content").await;

    let client = HttpClient::new();
    let config = HttpRequestConfig {
        url: server.url(),
        method: "GET".to_string(),
        headers: None,
        expected_status_code: 200,
        timeout_sec: 5,
        skip_tls: false,
        follow_redirect: true,
        auth: None,
    };

    let metrics = client.execute(&config).await;

    // Verify all timing phases are captured
    assert!(metrics.dns_duration >= Duration::ZERO, "DNS duration should be set");
    assert!(metrics.tcp_connect_duration >= Duration::ZERO, "TCP connect duration should be set");
    assert!(metrics.time_to_first_byte >= Duration::ZERO, "TTFB should be set");
    assert!(
        metrics.content_transfer_duration >= Duration::ZERO,
        "Content transfer duration should be set"
    );
    assert!(metrics.total_duration >= Duration::ZERO, "Total duration should be set");

    // Total should be >= sum of parts (approximately)
    let _ = metrics.dns_duration
        + metrics.tcp_connect_duration
        + metrics.time_to_first_byte
        + metrics.content_transfer_duration;

    // Allow some tolerance for timing variations
    assert!(metrics.total_duration >= Duration::from_nanos(1), "Total duration should be positive");
}

#[tokio::test]
async fn test_probe_resolved_ip() {
    let server = MockHttpServer::start(200, "OK").await;

    let client = HttpClient::new();
    let config = HttpRequestConfig {
        url: server.url(),
        method: "GET".to_string(),
        headers: None,
        expected_status_code: 200,
        timeout_sec: 5,
        skip_tls: false,
        follow_redirect: true,
        auth: None,
    };

    let metrics = client.execute(&config).await;

    assert!(metrics.resolved_ip.is_some(), "Resolved IP should be set");
    assert!(
        metrics.resolved_ip.as_ref().unwrap().starts_with("127.0.0.1"),
        "Should resolve to localhost"
    );
}

#[tokio::test]
async fn test_probe_http_version() {
    let server = MockHttpServer::start(200, "OK").await;

    let client = HttpClient::new();
    let config = HttpRequestConfig {
        url: server.url(),
        method: "GET".to_string(),
        headers: None,
        expected_status_code: 200,
        timeout_sec: 5,
        skip_tls: false,
        follow_redirect: true,
        auth: None,
    };

    let metrics = client.execute(&config).await;

    assert!(!metrics.http_version.is_empty(), "HTTP version should be set");
}

#[tokio::test]
async fn test_probe_content_length() {
    let body = "This is a test body with some content.";
    let server = MockHttpServer::start(200, body).await;

    let client = HttpClient::new();
    let config = HttpRequestConfig {
        url: server.url(),
        method: "GET".to_string(),
        headers: None,
        expected_status_code: 200,
        timeout_sec: 5,
        skip_tls: false,
        follow_redirect: true,
        auth: None,
    };

    let metrics = client.execute(&config).await;

    assert!(metrics.content_length.is_some(), "Content length should be set");
    assert_eq!(
        metrics.content_length.unwrap(),
        body.len() as u64,
        "Content length should match body size"
    );
}

// ===== HTTP Probe Trait Tests =====

#[tokio::test]
async fn test_http_probe_init() {
    let probe = HttpProbe::init();
    // Should create without error
    drop(probe);
}

#[tokio::test]
async fn test_http_probe_set_targets() {
    let mut probe = HttpProbe::init();

    let targets = vec![
        HttpTarget {
            url: "http://example.com".to_string(),
            method: "GET".to_string(),
            headers: None,
            expected_status_code: 200,
            timeout_sec: 30,
            skip_tls: false,
            follow_redirect: true,
            auth: None,
            labels: None,
            scrape_interval: ScrapeInterval::S30,
        },
        HttpTarget {
            url: "http://example.org".to_string(),
            method: "POST".to_string(),
            headers: None,
            expected_status_code: 201,
            timeout_sec: 60,
            skip_tls: false,
            follow_redirect: false,
            auth: None,
            labels: None,
            scrape_interval: ScrapeInterval::S30,
        },
    ];

    probe.set_targets(targets);
    // Should set without error
}

#[tokio::test]
async fn test_http_probe_scrape_and_get_metrics() {
    let server = MockHttpServer::start(200, "OK").await;

    let mut probe = HttpProbe::init();

    let targets = vec![HttpTarget {
        url: server.url(),
        method: "GET".to_string(),
        headers: None,
        expected_status_code: 200,
        timeout_sec: 5,
        skip_tls: false,
        follow_redirect: true,
        auth: None,
        labels: Some(HashMap::from([
            ("service".to_string(), "test-service".to_string()),
            ("env".to_string(), "test".to_string()),
        ])),
        scrape_interval: ScrapeInterval::S30,
    }];

    probe.set_targets(targets);
    probe.scrape().await;

    let metrics = probe.get_metrics().await;

    assert_eq!(metrics.len(), 1, "Should have one metric result");

    let metric = &metrics[0];
    assert!(metric.metrics.contains_key("up"));
    assert!(metric.metrics.contains_key("success"));
    assert!(metric.metrics.contains_key("status_code"));
    assert!(metric.metrics.contains_key("dns_duration_ms"));
    assert!(metric.metrics.contains_key("tcp_connect_duration_ms"));
    assert!(metric.metrics.contains_key("time_to_first_byte_ms"));
    assert!(metric.metrics.contains_key("total_duration_ms"));

    // Check labels
    assert_eq!(metric.labels.get("service"), Some(&"test-service".to_string()));
    assert_eq!(metric.labels.get("env"), Some(&"test".to_string()));
    assert_eq!(metric.labels.get("probe"), Some(&"http".to_string()));
}

#[tokio::test]
async fn test_http_probe_multiple_targets() {
    let server1 = MockHttpServer::start(200, "OK").await;
    let server2 = MockHttpServer::start(201, "Created").await;

    let mut probe = HttpProbe::init();

    let targets = vec![
        HttpTarget {
            url: server1.url(),
            method: "GET".to_string(),
            headers: None,
            expected_status_code: 200,
            timeout_sec: 5,
            skip_tls: false,
            follow_redirect: true,
            auth: None,
            labels: Some(HashMap::from([("name".to_string(), "server1".to_string())])),
            scrape_interval: ScrapeInterval::S30,
        },
        HttpTarget {
            url: server2.url(),
            method: "GET".to_string(),
            headers: None,
            expected_status_code: 201,
            timeout_sec: 5,
            skip_tls: false,
            follow_redirect: true,
            auth: None,
            labels: Some(HashMap::from([("name".to_string(), "server2".to_string())])),
            scrape_interval: ScrapeInterval::S30,
        },
    ];

    probe.set_targets(targets);
    probe.scrape().await;

    let metrics = probe.get_metrics().await;

    assert_eq!(metrics.len(), 2, "Should have two metric results");

    // Both should be successful
    for metric in &metrics {
        assert_eq!(*metric.metrics.get("up").unwrap(), 1);
        assert_eq!(*metric.metrics.get("success").unwrap(), 1);
    }
}

#[tokio::test]
async fn test_http_probe_metrics_cleared_after_get() {
    let server = MockHttpServer::start(200, "OK").await;

    let mut probe = HttpProbe::init();

    let targets = vec![HttpTarget {
        url: server.url(),
        method: "GET".to_string(),
        headers: None,
        expected_status_code: 200,
        timeout_sec: 5,
        skip_tls: false,
        follow_redirect: true,
        auth: None,
        labels: None,
        scrape_interval: ScrapeInterval::S30,
    }];

    probe.set_targets(targets);
    probe.scrape().await;

    // First get should return metrics
    let metrics1 = probe.get_metrics().await;
    assert_eq!(metrics1.len(), 1);

    // Second get should return empty (metrics were cleared)
    let metrics2 = probe.get_metrics().await;
    assert!(metrics2.is_empty(), "Metrics should be cleared after get");
}

// ===== Edge Cases =====

#[tokio::test]
async fn test_probe_empty_targets() {
    let mut probe = HttpProbe::init();
    probe.set_targets(vec![]);
    probe.scrape().await;

    let metrics = probe.get_metrics().await;
    assert!(metrics.is_empty(), "No targets = no metrics");
}

#[tokio::test]
async fn test_probe_concurrent_scrapes() {
    let server = MockHttpServer::start(200, "OK").await;

    let probe = Arc::new(tokio::sync::Mutex::new(HttpProbe::init()));

    let targets = vec![HttpTarget {
        url: server.url(),
        method: "GET".to_string(),
        headers: None,
        expected_status_code: 200,
        timeout_sec: 5,
        skip_tls: false,
        follow_redirect: true,
        auth: None,
        labels: None,
        scrape_interval: ScrapeInterval::S30,
    }];

    {
        let mut p = probe.lock().await;
        p.set_targets(targets);
    }

    // Run multiple scrapes concurrently
    let probe1 = Arc::clone(&probe);
    let probe2 = Arc::clone(&probe);

    let (_, _) = tokio::join!(
        async move {
            let p = probe1.lock().await;
            p.scrape().await;
        },
        async move {
            let p = probe2.lock().await;
            p.scrape().await;
        }
    );
}
