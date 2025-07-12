pub mod defaults;
pub mod discovery;
pub mod exporter;
pub mod scrape_interval;
pub mod target;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Configuration {
    pub defaults: defaults::Defaults,
    pub discovery: Option<discovery::Discovery>,
    pub http: Option<target::HttpConfiguration>,
    pub icmp: Option<target::IcmpConfiguration>,
    pub exporter: exporter::ExporterConfiguration,
}
