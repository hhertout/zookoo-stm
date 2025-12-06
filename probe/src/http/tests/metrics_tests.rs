//! Unit tests for HttpProbeMetrics

use super::super::metrics::HttpProbeMetrics;
use std::time::Duration;

#[test]
fn test_new_creates_default_metrics() {
    let metrics = HttpProbeMetrics::new();

    assert_eq!(metrics.dns_duration, Duration::ZERO);
    assert_eq!(metrics.tcp_connect_duration, Duration::ZERO);
    assert!(metrics.tls_handshake_duration.is_none());
    assert_eq!(metrics.time_to_first_byte, Duration::ZERO);
    assert_eq!(metrics.content_transfer_duration, Duration::ZERO);
    assert_eq!(metrics.total_duration, Duration::ZERO);
    assert!(metrics.tls_version.is_none());
    assert!(metrics.cert_expiration_ts.is_none());
    assert!(metrics.cert_begin_ts.is_none());
    assert!(metrics.cert_issuer.is_none());
    assert!(metrics.cert_subject.is_none());
    assert_eq!(metrics.status_code, 0);
    assert!(metrics.http_version.is_empty());
    assert!(metrics.content_length.is_none());
    assert!(!metrics.up);
    assert!(!metrics.success);
    assert!(metrics.resolved_ip.is_none());
}

#[test]
fn test_default_equals_new() {
    let new_metrics = HttpProbeMetrics::new();
    let default_metrics = HttpProbeMetrics::default();

    assert_eq!(new_metrics.up, default_metrics.up);
    assert_eq!(new_metrics.success, default_metrics.success);
    assert_eq!(new_metrics.status_code, default_metrics.status_code);
    assert_eq!(new_metrics.dns_duration, default_metrics.dns_duration);
}

#[test]
fn test_failed_creates_empty_metrics() {
    let metrics = HttpProbeMetrics::failed();

    assert!(!metrics.up);
    assert!(!metrics.success);
    assert_eq!(metrics.status_code, 0);
}

#[test]
fn test_dns_failed_preserves_dns_duration() {
    let dns_time = Duration::from_millis(50);
    let metrics = HttpProbeMetrics::dns_failed(dns_time);

    assert_eq!(metrics.dns_duration, dns_time);
    assert!(!metrics.up);
    assert!(!metrics.success);
    assert_eq!(metrics.status_code, 0);
}

#[test]
fn test_to_logfmt_basic() {
    let metrics = HttpProbeMetrics::new();
    let logfmt = metrics.to_logfmt();

    assert!(logfmt.contains("up=0"));
    assert!(logfmt.contains("success=0"));
    assert!(logfmt.contains("status_code=0"));
    assert!(logfmt.contains("dns_ms=0"));
    assert!(logfmt.contains("tcp_ms=0"));
    assert!(logfmt.contains("ttfb_ms=0"));
    assert!(logfmt.contains("total_ms=0"));
}

#[test]
fn test_to_logfmt_with_tls() {
    let mut metrics = HttpProbeMetrics::new();
    metrics.tls_handshake_duration = Some(Duration::from_millis(100));
    metrics.tls_version = Some("TLSv1.3".to_string());

    let logfmt = metrics.to_logfmt();

    assert!(logfmt.contains("tls_handshake_ms=100"));
    assert!(logfmt.contains("tls_version=TLSv1.3"));
}

#[test]
fn test_to_logfmt_with_resolved_ip() {
    let mut metrics = HttpProbeMetrics::new();
    metrics.resolved_ip = Some("192.168.1.1".to_string());

    let logfmt = metrics.to_logfmt();

    assert!(logfmt.contains("resolved_ip=192.168.1.1"));
}

#[test]
fn test_to_logfmt_with_success() {
    let mut metrics = HttpProbeMetrics::new();
    metrics.up = true;
    metrics.success = true;
    metrics.status_code = 200;

    let logfmt = metrics.to_logfmt();

    assert!(logfmt.contains("up=1"));
    assert!(logfmt.contains("success=1"));
    assert!(logfmt.contains("status_code=200"));
}

#[test]
fn test_to_metrics_map_basic() {
    let metrics = HttpProbeMetrics::new();
    let map = metrics.to_metrics_map();

    assert_eq!(*map.get("up").unwrap(), 0);
    assert_eq!(*map.get("success").unwrap(), 0);
    assert_eq!(*map.get("status_code").unwrap(), 0);
    assert_eq!(*map.get("dns_duration_ms").unwrap(), 0);
    assert_eq!(*map.get("tcp_connect_duration_ms").unwrap(), 0);
    assert_eq!(*map.get("time_to_first_byte_ms").unwrap(), 0);
    assert_eq!(*map.get("content_transfer_duration_ms").unwrap(), 0);
    assert_eq!(*map.get("total_duration_ms").unwrap(), 0);
}

#[test]
fn test_to_metrics_map_with_durations() {
    let mut metrics = HttpProbeMetrics::new();
    metrics.dns_duration = Duration::from_millis(10);
    metrics.tcp_connect_duration = Duration::from_millis(20);
    metrics.time_to_first_byte = Duration::from_millis(30);
    metrics.content_transfer_duration = Duration::from_millis(40);
    metrics.total_duration = Duration::from_millis(100);

    let map = metrics.to_metrics_map();

    assert_eq!(*map.get("dns_duration_ms").unwrap(), 10);
    assert_eq!(*map.get("tcp_connect_duration_ms").unwrap(), 20);
    assert_eq!(*map.get("time_to_first_byte_ms").unwrap(), 30);
    assert_eq!(*map.get("content_transfer_duration_ms").unwrap(), 40);
    assert_eq!(*map.get("total_duration_ms").unwrap(), 100);
}

#[test]
fn test_to_metrics_map_with_tls_data() {
    let mut metrics = HttpProbeMetrics::new();
    metrics.tls_handshake_duration = Some(Duration::from_millis(50));
    metrics.cert_expiration_ts = Some(1735689600); // 2025-01-01
    metrics.cert_begin_ts = Some(1704067200); // 2024-01-01

    let map = metrics.to_metrics_map();

    assert_eq!(*map.get("tls_handshake_ms").unwrap(), 50);
    assert_eq!(*map.get("cert_expiration_ts").unwrap(), 1735689600);
    assert_eq!(*map.get("cert_begin_ts").unwrap(), 1704067200);
}

#[test]
fn test_to_metrics_map_without_optional_fields() {
    let metrics = HttpProbeMetrics::new();
    let map = metrics.to_metrics_map();

    assert!(!map.contains_key("tls_handshake_ms"));
    assert!(!map.contains_key("cert_expiration_ts"));
    assert!(!map.contains_key("cert_begin_ts"));
}

#[test]
fn test_metrics_success_status() {
    let mut metrics = HttpProbeMetrics::new();
    metrics.up = true;
    metrics.success = true;
    metrics.status_code = 200;

    let map = metrics.to_metrics_map();

    assert_eq!(*map.get("up").unwrap(), 1);
    assert_eq!(*map.get("success").unwrap(), 1);
    assert_eq!(*map.get("status_code").unwrap(), 200);
}

#[test]
fn test_metrics_failure_status() {
    let mut metrics = HttpProbeMetrics::new();
    metrics.up = true;
    metrics.success = false;
    metrics.status_code = 500;

    let map = metrics.to_metrics_map();

    assert_eq!(*map.get("up").unwrap(), 1);
    assert_eq!(*map.get("success").unwrap(), 0);
    assert_eq!(*map.get("status_code").unwrap(), 500);
}

#[test]
fn test_clone_preserves_all_fields() {
    let mut metrics = HttpProbeMetrics::new();
    metrics.dns_duration = Duration::from_millis(10);
    metrics.tcp_connect_duration = Duration::from_millis(20);
    metrics.tls_handshake_duration = Some(Duration::from_millis(30));
    metrics.time_to_first_byte = Duration::from_millis(40);
    metrics.content_transfer_duration = Duration::from_millis(50);
    metrics.total_duration = Duration::from_millis(150);
    metrics.tls_version = Some("TLSv1.3".to_string());
    metrics.cert_expiration_ts = Some(1735689600);
    metrics.cert_begin_ts = Some(1704067200);
    metrics.cert_issuer = Some("Test CA".to_string());
    metrics.cert_subject = Some("example.com".to_string());
    metrics.status_code = 200;
    metrics.http_version = "HTTP/1.1".to_string();
    metrics.content_length = Some(1024);
    metrics.up = true;
    metrics.success = true;
    metrics.resolved_ip = Some("1.2.3.4".to_string());

    let cloned = metrics.clone();

    assert_eq!(cloned.dns_duration, metrics.dns_duration);
    assert_eq!(cloned.tcp_connect_duration, metrics.tcp_connect_duration);
    assert_eq!(cloned.tls_handshake_duration, metrics.tls_handshake_duration);
    assert_eq!(cloned.time_to_first_byte, metrics.time_to_first_byte);
    assert_eq!(cloned.content_transfer_duration, metrics.content_transfer_duration);
    assert_eq!(cloned.total_duration, metrics.total_duration);
    assert_eq!(cloned.tls_version, metrics.tls_version);
    assert_eq!(cloned.cert_expiration_ts, metrics.cert_expiration_ts);
    assert_eq!(cloned.cert_begin_ts, metrics.cert_begin_ts);
    assert_eq!(cloned.cert_issuer, metrics.cert_issuer);
    assert_eq!(cloned.cert_subject, metrics.cert_subject);
    assert_eq!(cloned.status_code, metrics.status_code);
    assert_eq!(cloned.http_version, metrics.http_version);
    assert_eq!(cloned.content_length, metrics.content_length);
    assert_eq!(cloned.up, metrics.up);
    assert_eq!(cloned.success, metrics.success);
    assert_eq!(cloned.resolved_ip, metrics.resolved_ip);
}
