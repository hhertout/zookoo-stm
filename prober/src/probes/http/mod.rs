//! HTTP Probe Module
//!
//! This module handles HTTP/HTTPS probing including:
//! - DNS resolution
//! - TLS certificate inspection
//! - HTTP request/response metrics
//! - Custom headers and authentication

pub mod dns;
pub mod metrics;
pub mod request;
pub mod scraper;
pub mod tls;

pub use scraper::HttpScraper;

// Re-export target type from config
pub use crate::config::target::HttpTarget;
