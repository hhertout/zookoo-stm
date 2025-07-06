use crate::config::target::{HttpConfiguration, IcmpConfiguration};

pub mod exporter;
pub mod scrape_interval;
pub mod target;

#[derive(Debug, Clone)]
pub struct ScrapConfiguration {
    pub http: Option<target::HttpConfiguration>,
    pub icmp: Option<target::IcmpConfiguration>,
    pub exporter: exporter::ExporterConfiguration,
}

impl From<configuration::model::Configuration> for ScrapConfiguration {
    fn from(value: configuration::model::Configuration) -> Self {
        ScrapConfiguration {
            http: value.http.map(HttpConfiguration::from),
            icmp: value.icmp.map(IcmpConfiguration::from),
            exporter: value.exporter.into(),
        }
    }
}
