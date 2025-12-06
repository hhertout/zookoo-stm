//! HTTP Probe Metrics
//!
//! Contains all timing and metadata metrics collected during a single HTTP probe.

use std::time::Duration;

/// Complete metrics collected from a single HTTP probe request.
///
/// All timings are measured on the same connection, providing accurate
/// phase-by-phase breakdown of the request lifecycle.
#[derive(Debug, Clone)]
pub struct HttpProbeMetrics {
    // === Timing Phases ===
    /// Time taken for DNS resolution
    pub dns_duration: Duration,
    /// Time taken to establish TCP connection
    pub tcp_connect_duration: Duration,
    /// Time taken for TLS handshake (None for HTTP)
    pub tls_handshake_duration: Option<Duration>,
    /// Time from request sent to first byte received (TTFB)
    pub time_to_first_byte: Duration,
    /// Time to transfer the response body
    pub content_transfer_duration: Duration,
    /// Total end-to-end duration
    pub total_duration: Duration,

    // === TLS Information ===
    /// TLS protocol version (e.g., "TLSv1.3")
    pub tls_version: Option<String>,
    /// Certificate expiration timestamp (Unix epoch seconds)
    pub cert_expiration_ts: Option<i64>,
    /// Certificate start validity timestamp (Unix epoch seconds)
    pub cert_begin_ts: Option<i64>,
    /// Certificate issuer common name
    pub cert_issuer: Option<String>,
    /// Certificate subject common name
    pub cert_subject: Option<String>,

    // === HTTP Information ===
    /// HTTP response status code
    pub status_code: u16,
    /// HTTP protocol version (e.g., "HTTP/1.1", "HTTP/2")
    pub http_version: String,
    /// Response content length (if available)
    pub content_length: Option<u64>,

    // === Probe Status ===
    /// Whether the target is reachable (TCP connection succeeded)
    pub up: bool,
    /// Whether the probe was successful (status code matches expected)
    pub success: bool,

    // === Target Information ===
    /// The resolved IP address used for the connection
    pub resolved_ip: Option<String>,
}

impl HttpProbeMetrics {
    /// Create a new metrics instance with default/zero values
    pub fn new() -> Self {
        Self {
            dns_duration: Duration::ZERO,
            tcp_connect_duration: Duration::ZERO,
            tls_handshake_duration: None,
            time_to_first_byte: Duration::ZERO,
            content_transfer_duration: Duration::ZERO,
            total_duration: Duration::ZERO,
            tls_version: None,
            cert_expiration_ts: None,
            cert_begin_ts: None,
            cert_issuer: None,
            cert_subject: None,
            status_code: 0,
            http_version: String::new(),
            content_length: None,
            up: false,
            success: false,
            resolved_ip: None,
        }
    }

    /// Create metrics for a failed probe (target unreachable)
    pub fn failed() -> Self {
        Self::new()
    }

    /// Create metrics for a DNS resolution failure
    pub fn dns_failed(dns_duration: Duration) -> Self {
        Self { dns_duration, ..Self::new() }
    }

    /// Convert metrics to logfmt format for structured logging
    pub fn to_logfmt(&self) -> String {
        let mut parts = vec![
            format!("up={}", self.up as u8),
            format!("success={}", self.success as u8),
            format!("status_code={}", self.status_code),
            format!("dns_ms={}", self.dns_duration.as_millis()),
            format!("tcp_ms={}", self.tcp_connect_duration.as_millis()),
            format!("ttfb_ms={}", self.time_to_first_byte.as_millis()),
            format!("total_ms={}", self.total_duration.as_millis()),
        ];

        if let Some(tls_dur) = &self.tls_handshake_duration {
            parts.push(format!("tls_handshake_ms={}", tls_dur.as_millis()));
        }

        if let Some(version) = &self.tls_version {
            parts.push(format!("tls_version={}", version));
        }

        if let Some(ip) = &self.resolved_ip {
            parts.push(format!("resolved_ip={}", ip));
        }

        parts.push(format!("http_version={}", self.http_version));

        parts.join(" ")
    }

    /// Convert to a HashMap for metric export
    /// Keys are named to match what the OTEL exporter expects
    pub fn to_metrics_map(&self) -> std::collections::HashMap<String, isize> {
        let mut map = std::collections::HashMap::new();

        map.insert("up".to_string(), self.up as isize);
        map.insert("success".to_string(), self.success as isize);
        map.insert("status_code".to_string(), self.status_code as isize);
        map.insert("dns_duration_ms".to_string(), self.dns_duration.as_millis() as isize);
        map.insert(
            "tcp_connect_duration_ms".to_string(),
            self.tcp_connect_duration.as_millis() as isize,
        );
        map.insert(
            "time_to_first_byte_ms".to_string(),
            self.time_to_first_byte.as_millis() as isize,
        );
        map.insert(
            "content_transfer_duration_ms".to_string(),
            self.content_transfer_duration.as_millis() as isize,
        );

        // http_duration_ms = total duration (for backward compatibility with exporter)
        map.insert("http_duration_ms".to_string(), self.total_duration.as_millis() as isize);
        map.insert("total_duration_ms".to_string(), self.total_duration.as_millis() as isize);

        if let Some(tls_dur) = &self.tls_handshake_duration {
            // tls_handshake_ms key expected by exporter
            map.insert("tls_handshake_ms".to_string(), tls_dur.as_millis() as isize);
        }

        if let Some(exp) = self.cert_expiration_ts {
            map.insert("cert_expiration_ts".to_string(), exp as isize);
        }

        if let Some(begin) = self.cert_begin_ts {
            map.insert("cert_begin_ts".to_string(), begin as isize);
        }

        map
    }
}

impl Default for HttpProbeMetrics {
    fn default() -> Self {
        Self::new()
    }
}
