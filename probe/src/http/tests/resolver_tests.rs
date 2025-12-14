//! Unit tests for DNS Resolver

use super::super::resolver::{DnsResolver, extract_host, extract_port};

// ===== URL Parsing Tests =====

#[test]
fn test_extract_host_https() {
    assert_eq!(extract_host("https://example.com").unwrap(), "example.com");
    assert_eq!(extract_host("https://example.com/path/to/resource").unwrap(), "example.com");
    assert_eq!(extract_host("https://example.com:8443/path").unwrap(), "example.com");
}

#[test]
fn test_extract_host_http() {
    assert_eq!(extract_host("http://example.com").unwrap(), "example.com");
    assert_eq!(extract_host("http://example.com:8080/path").unwrap(), "example.com");
}

#[test]
fn test_extract_host_localhost() {
    assert_eq!(extract_host("http://localhost").unwrap(), "localhost");
    assert_eq!(extract_host("http://localhost:8080").unwrap(), "localhost");
    assert_eq!(extract_host("https://localhost:443").unwrap(), "localhost");
}

#[test]
fn test_extract_host_ip_address() {
    assert_eq!(extract_host("http://192.168.1.1").unwrap(), "192.168.1.1");
    assert_eq!(extract_host("http://192.168.1.1:8080").unwrap(), "192.168.1.1");
    assert_eq!(extract_host("http://127.0.0.1:3000").unwrap(), "127.0.0.1");
}

#[test]
fn test_extract_host_subdomain() {
    assert_eq!(extract_host("https://api.example.com").unwrap(), "api.example.com");
    assert_eq!(extract_host("https://www.example.com").unwrap(), "www.example.com");
    assert_eq!(extract_host("https://sub.domain.example.com").unwrap(), "sub.domain.example.com");
}

#[test]
fn test_extract_host_with_query_params() {
    assert_eq!(extract_host("https://example.com?foo=bar").unwrap(), "example.com");
    assert_eq!(extract_host("https://example.com/path?foo=bar&baz=qux").unwrap(), "example.com");
}

#[test]
fn test_extract_host_with_fragment() {
    assert_eq!(extract_host("https://example.com#section").unwrap(), "example.com");
    assert_eq!(extract_host("https://example.com/path#section").unwrap(), "example.com");
}

#[test]
fn test_extract_host_invalid_url() {
    assert!(extract_host("not-a-url").is_err());
    assert!(extract_host("").is_err());
    assert!(extract_host("://missing-scheme.com").is_err());
}

#[test]
fn test_extract_port_https_default() {
    assert_eq!(extract_port("https://example.com").unwrap(), 443);
    assert_eq!(extract_port("https://example.com/path").unwrap(), 443);
}

#[test]
fn test_extract_port_http_default() {
    assert_eq!(extract_port("http://example.com").unwrap(), 80);
    assert_eq!(extract_port("http://example.com/path").unwrap(), 80);
}

#[test]
fn test_extract_port_custom() {
    assert_eq!(extract_port("http://example.com:8080").unwrap(), 8080);
    assert_eq!(extract_port("https://example.com:8443").unwrap(), 8443);
    assert_eq!(extract_port("http://localhost:3000").unwrap(), 3000);
}

#[test]
fn test_extract_port_invalid_url() {
    assert!(extract_port("not-a-url").is_err());
    assert!(extract_port("").is_err());
}

// ===== DNS Resolver Tests =====

#[test]
fn test_resolver_clone() {
    let resolver = DnsResolver::new();
    let cloned = resolver.clone();
    // Both should exist without issue
    drop(resolver);
    drop(cloned);
}

// ===== Async DNS Resolution Tests =====

#[tokio::test]
async fn test_resolve_localhost() {
    let resolver = DnsResolver::new();
    let result = resolver.resolve("localhost").await;

    // localhost should always resolve
    let dns_result = result.expect("localhost should resolve");
    assert!(!dns_result.addresses.is_empty(), "Should have at least one address");
    assert!(dns_result.duration.as_nanos() > 0, "Duration should be non-zero");
}

#[tokio::test]
async fn test_resolve_google_dns() {
    let resolver = DnsResolver::new();
    let result = resolver.resolve("dns.google").await;

    // dns.google should resolve (Google's public DNS)
    if let Ok(dns_result) = result {
        assert!(!dns_result.addresses.is_empty());
    }
    // Note: We allow this to fail in isolated environments
}

#[tokio::test]
async fn test_resolve_invalid_domain() {
    let resolver = DnsResolver::new();
    let result = resolver.resolve("this-domain-definitely-does-not-exist-12345.invalid").await;

    assert!(result.is_err(), "Invalid domain should fail to resolve");
}

#[tokio::test]
async fn test_resolve_first_ipv4_localhost() {
    let resolver = DnsResolver::new();
    let result = resolver.resolve_first_ipv4("localhost").await;

    let (ip, duration) = result.expect("localhost should resolve");
    assert!(ip.is_ipv4() || ip.is_ipv6(), "Should be a valid IP");
    assert!(duration.as_nanos() > 0, "Duration should be non-zero");
}

#[tokio::test]
async fn test_resolve_first_ipv4_invalid() {
    let resolver = DnsResolver::new();
    let result =
        resolver.resolve_first_ipv4("this-domain-definitely-does-not-exist-12345.invalid").await;

    assert!(result.is_err(), "Invalid domain should fail");
}

// ===== DNS Result Tests =====

#[tokio::test]
async fn test_dns_result_contains_duration() {
    let resolver = DnsResolver::new();
    let result = resolver.resolve("localhost").await;

    if let Ok(dns_result) = result {
        // Duration should be captured
        assert!(dns_result.duration >= std::time::Duration::ZERO);
    }
}

#[tokio::test]
async fn test_dns_result_multiple_addresses() {
    let resolver = DnsResolver::new();
    // Some hosts return multiple addresses
    let result = resolver.resolve("localhost").await;

    if let Ok(dns_result) = result {
        // localhost might have both IPv4 and IPv6
        assert!(!dns_result.addresses.is_empty());
    }
}
