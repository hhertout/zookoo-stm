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

/// Parse trait for configuration files.
/// This trait defines a method to parse a configuration file from a given file path.
/// It is generic over the type `T`, allowing for flexibility in the types of configurations that can be parsed.
/// The `parse_from_file` method reads the content of the file and deserializes it into the specified type `T`.
/// It returns a result containing the deserialized value or an error if the operation fails.
/// This trait can be implemented for various configuration types, enabling easy parsing
/// of configuration files in different formats, such as JSON or TOML.
pub trait Parse<T> {
    /// Parse a configuration file from the given file path.
    fn parse_from_file(&self, file_path: &str) -> Result<T, Box<dyn Error>>;
}

/// Discovery trait for configuration files.
/// This trait defines a method to fetch discovery information and update the configuration.
/// It is generic over the type `T`, allowing for flexibility in the types of configurations that can be updated.
/// The `fetch_discovery` method retrieves the discovery configuration and processes it to update
/// the main configuration. It returns a result indicating success or failure of the operation.
/// This trait can be implemented for various discovery mechanisms, enabling dynamic updates to the configuration
/// based on external discovery files or services.
pub trait Discovery<T> {
    /// Fetch discovery information and update the configuration.
    fn fetch_discovery(&self, configuration: &mut T) -> Result<(), Box<dyn Error>>;
}

pub struct ConfigParser;
impl ConfigParser {
    pub fn new() -> Self {
        ConfigParser {}
    }

    /// Deserialize a JSON file into the specified type.
    /// This method reads the content of the file at the given path and deserializes it
    /// into the specified type `T`. It returns a result containing the deserialized value or
    /// an error if the operation fails.
    fn discovery_into<T>(&self, path: &str) -> Result<T, Box<dyn Error>>
    where
        T: DeserializeOwned,
    {
        let content = fs::read_to_string(path)?;
        let res: T = serde_json::from_str(&content)?;
        Ok(res)
    }

    /// Override HTTP target parameters based on the discovery configuration.
    /// This method modifies the scrape interval and labels of the HTTP targets based on the provided discovery
    /// configuration. If the discovery configuration specifies a scrape interval or labels, those values
    /// will be applied to all parsed HTTP targets.
    /// It iterates over the parsed targets and updates their scrape interval and labels accordingly.
    /// If the discovery configuration does not specify these values, the targets remain unchanged.
    /// This allows for dynamic configuration of HTTP targets based on external discovery files.
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
    /// Fetch discovery information and update the configuration.
    /// This method retrieves the discovery configuration from the provided `Configuration` object,
    /// processes the HTTP targets defined in the discovery configuration, and updates the main configuration
    /// with the parsed HTTP targets. It reads the discovery files specified in the configuration,
    /// applies any overrides specified in the discovery configuration, and appends the parsed targets to the
    /// existing HTTP targets in the configuration.
    /// If the discovery configuration is not present or does not specify any HTTP targets, it does not modify the configuration.
    /// This allows for dynamic discovery of HTTP targets based on external files,
    /// enabling the application to adapt to changes in the target environment without requiring a restart.
    /// It returns a result indicating success or failure of the operation.
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
    /// Parse a configuration file from the given file path.
    /// This method reads the content of the file at the specified path and deserializes it
    /// into a `Configuration` object. It uses the `serde` library to perform the deserialization.
    /// If the file is successfully read and parsed, it returns a `Configuration` object.
    /// If there is an error during reading or parsing, it returns an error wrapped in a `Box<dyn Error>`.
    /// This allows for flexible error handling and makes it easy to integrate with other parts of the application.
    fn parse_from_file<'a>(&self, file_path: &'a str) -> Result<Configuration, Box<dyn Error>> {
        let content = fs::read_to_string(file_path)?;
        let conf = toml::from_str(&content)?;

        Ok(conf)
    }
}
