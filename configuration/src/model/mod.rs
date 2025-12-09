pub mod defaults;
pub mod discovery;
pub mod exporter;
pub mod probe;
pub mod scrape_interval;
pub mod target;

use serde::Deserialize;
use std::collections::HashMap;

// Re-export HasScrapeInterval trait
pub use scrape_interval::HasScrapeInterval;
pub use scrape_interval::ScrapeInterval;

#[derive(Debug, Deserialize)]
pub struct Configuration {
    pub defaults: defaults::Defaults,
    pub probe: Option<ProbeWrapper>,
    pub exporter: Option<ExporterWrapper>,
    pub discovery: Option<DiscoveryWrapper>,
}

#[derive(Debug, Deserialize)]
pub struct ProbeWrapper {
    #[serde(default)]
    pub http: HashMap<String, target::HttpConfiguration>,
    #[serde(default)]
    pub icmp: HashMap<String, target::IcmpConfiguration>,
}

#[derive(Debug, Deserialize)]
pub struct ExporterWrapper {
    #[serde(default)]
    pub otlp: HashMap<String, exporter::OtelGrpcExporterConfiguration>,
    #[serde(default)]
    pub prometheus_remote_write: HashMap<String, exporter::PrometheusRemoteWriteConfiguration>,
    #[serde(default)]
    pub timescale: HashMap<String, exporter::TimescaleExporterConfiguration>,
}

#[derive(Debug, Deserialize)]
pub struct DiscoveryWrapper {
    #[serde(default)]
    pub file: HashMap<String, discovery::DiscoveryFile>,
}
