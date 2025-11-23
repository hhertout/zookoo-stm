//! ICMP Probe Module
//!
//! This module handles ICMP (ping) probing including:
//! - IPv4 address resolution
//! - FQDN to IP resolution
//! - Ping latency measurements

pub mod metrics;
pub mod ping;
pub mod scraper;
#[cfg(test)]
mod ping_tests;

pub use scraper::IcmpScraper;

// Re-export target type from config
pub use crate::config::target::IcmpTarget;
