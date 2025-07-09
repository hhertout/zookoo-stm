use crate::config::target::{HttpConfiguration, IcmpConfiguration};

pub(crate) mod defaults;
pub(crate) mod exporter;
pub(crate) mod scrape_interval;
pub(crate) mod target;

#[derive(Debug, Clone)]
pub struct ScrapConfiguration {
    pub default: defaults::Defaults,
    pub http: Option<target::HttpConfiguration>,
    pub icmp: Option<target::IcmpConfiguration>,
    pub exporter: exporter::ExporterConfiguration,
}

impl From<configuration::model::Configuration> for ScrapConfiguration {
    fn from(value: configuration::model::Configuration) -> Self {
        ScrapConfiguration {
            default: defaults::Defaults::from(value.defaults),
            http: value.http.map(HttpConfiguration::from),
            icmp: value.icmp.map(IcmpConfiguration::from),
            exporter: value.exporter.into(),
        }
    }
}
