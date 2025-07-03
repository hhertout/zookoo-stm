use crate::config::target::HttpConfiguration;

pub mod exporter;
pub mod scrap_interval;
pub mod target;

#[derive(Debug, Clone)]
pub struct ScrapConfiguration {
    pub http: Option<target::HttpConfiguration>,
    pub exporter: exporter::ExporterConfiguration,
}

impl From<configuration::model::Configuration> for ScrapConfiguration {
    fn from(value: configuration::model::Configuration) -> Self {
        ScrapConfiguration {
            http: value.http.map(HttpConfiguration::from),
            exporter: value.exporter.into(),
        }
    }
}
