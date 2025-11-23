//! Probes Module
//!
//! This module contains all probe type implementations.
//! Each probe type is self-contained with its own:
//! - Configuration (target type)
//! - Scraper implementation
//! - Metrics implementation
//! - Helper modules
//!
//! To add a new probe type:
//! 1. Create a new directory under probes/
//! 2. Implement the Scraping trait
//! 3. Implement MetricExportable for your metrics
//! 4. Add the module here

pub mod http;
pub mod icmp;

// Re-export commonly used types
pub use http::{HttpScraper, HttpTarget};
pub use icmp::{IcmpScraper, IcmpTarget};
