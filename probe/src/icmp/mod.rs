//! ICMP Probe Module
//!
//! This module handles ICMP (ping) probing including:
//! - IPv4 address resolution
//! - FQDN to IP resolution
//! - Ping latency measurements

pub mod metrics;
pub mod ping;
pub mod probe;

pub use probe::IcmpProbe;

#[cfg(test)]
mod ping_tests;
