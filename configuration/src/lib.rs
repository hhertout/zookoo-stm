//! # Configuration crate
//!
//! This crate is responsible of the parsing of the configuration file
//!

use serde::de::DeserializeOwned;
use std::error::Error;
use std::fs;

use crate::model::Configuration;
use crate::model::discovery::DiscoveryFileTarget;
use crate::model::target::{HttpConfiguration, HttpTarget};

pub mod model;

pub trait Parse<T> {
    fn parse_from_file(&self, file_path: &str) -> Result<T, Box<dyn Error>>;
}

pub trait Discovery<T> {
    fn fetch_discovery(&self, configuration: &mut T) -> Result<(), Box<dyn Error>>;
}

pub struct ConfigParser;
impl ConfigParser {
    pub fn new() -> Self {
        ConfigParser {}
    }

    fn discovery_into<T>(&self, path: &str) -> Result<T, Box<dyn Error>>
    where
        T: DeserializeOwned,
    {
        let content = fs::read_to_string(path)?;
        let res: T = serde_json::from_str(&content)?;
        Ok(res)
    }

    fn override_http_params(
        &self,
        parsed_targets: &mut Vec<HttpTarget>,
        discovery_conf: &DiscoveryFileTarget,
    ) {
        if let Some(override_scrape_interval) = &discovery_conf.scrape_interval {
            for target in parsed_targets.iter_mut() {
                target.scrape_interval = override_scrape_interval.clone();
            }
        }

        if let Some(override_labels) = &discovery_conf.labels {
            for target in parsed_targets.iter_mut() {
                if let Some(base_labels) = &mut target.labels {
                    base_labels.extend(override_labels.clone());
                } else {
                    target.labels = Some(override_labels.clone());
                }
            }
        }
    }
}

impl Discovery<Configuration> for ConfigParser {
    fn fetch_discovery(&self, configuration: &mut Configuration) -> Result<(), Box<dyn Error>> {
        let http_conf = configuration
            .discovery
            .as_ref()
            .and_then(|d| d.file.as_ref())
            .and_then(|f| f.http.as_ref());

        if let Some(http) = http_conf {
            for http_conf in http {
                for file_path in &http_conf.path {
                    let mut parsed_conf = self.discovery_into::<Vec<HttpTarget>>(&file_path)?;

                    self.override_http_params(&mut parsed_conf, http_conf);
                    configuration
                        .http
                        .get_or_insert_with(|| HttpConfiguration { targets: vec![] })
                        .targets
                        .append(&mut parsed_conf);
                }
            }
        }

        Ok(())
    }
}

impl Parse<Configuration> for ConfigParser {
    fn parse_from_file<'a>(&self, file_path: &'a str) -> Result<Configuration, Box<dyn Error>> {
        let content = fs::read_to_string(file_path)?;
        let conf = toml::from_str(&content)?;

        Ok(conf)
    }
}
