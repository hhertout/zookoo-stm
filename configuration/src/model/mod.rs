pub mod defaults;
pub mod exporter;
pub mod scrap_interval;
pub mod target;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Configuration {
    pub defaults: defaults::Defaults,
    pub http: Option<target::HttpConfiguration>,
    pub exporter: exporter::ExporterConfiguration,
}
