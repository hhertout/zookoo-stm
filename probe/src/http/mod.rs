//! HTTP Probe Module
//!
//! This module handles HTTP/HTTPS probing with unified phase timing:
//! - DNS resolution
//! - TCP connection
//! - TLS handshake (for HTTPS)
//! - HTTP request/response with TTFB and content transfer timing
//!
//! All phases are measured on a single connection for accurate metrics.

mod client;
mod metrics;
mod probe;
mod resolver;
mod tls;

#[cfg(test)]
mod tests;

pub use client::{AuthConfig, HttpClient, HttpRequestConfig};
pub use metrics::HttpProbeMetrics;
pub use probe::{HttpProbe, TargetType};
pub use resolver::{DnsResolver, DnsResult, extract_host, extract_port};
pub use tls::{CertInfo, TlsHandler, TlsResult};
