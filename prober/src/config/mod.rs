///! This module defines the configuration for scraping targets in the prober application.
///! It includes the structure for the scrap configuration, default settings, and methods to group targets by their scrape intervals.
///! The configuration is used to define how the scraping should be performed, including intervals and labels.
use crate::config::target::{HttpConfiguration, IcmpConfiguration};

pub(crate) mod defaults;
pub(crate) mod exporter;
pub(crate) mod scrape_interval;
pub(crate) mod target;

/// ScrapConfiguration is a struct that holds the configuration for scraping targets.
/// It includes default settings, HTTP targets, ICMP targets, and exporter configurations.
/// The configuration is used to define how the scraping should be performed, including intervals and labels.
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
