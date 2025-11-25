///! This module defines the default configuration settings for the prober application.
///! It includes the structure for default settings, such as probe location, zone, and self-monitoring configurations.
///! The defaults are used to initialize the application with sensible values if not explicitly provided in the configuration file.
use std::collections::HashMap;

use serde::Deserialize;

fn default_self_monitoring_enabled() -> bool {
    return false;
}

fn default_tls_ignore() -> bool {
    return false;
}

fn default_self_monitoring_otel_endpoint() -> String {
    return String::from("http://localhost:4317");
}

fn default_self_monitoring_service_name() -> String {
    return String::from("zookoo");
}

fn default_self_monitoring_env() -> String {
    return String::from("development");
}

#[derive(Debug, Clone, Deserialize)]
pub struct Defaults {
    pub probe_location: Option<ProbeLocation>,
    pub probe_zone: Option<String>,
    pub self_monitoring: Option<SelfMonitoringConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProbeLocation {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SelfMonitoringConfig {
    #[serde(default = "default_self_monitoring_enabled")]
    pub enable: bool,
    #[serde(default = "default_self_monitoring_otel_endpoint")]
    pub otel_endpoint: String,
    #[serde(default = "default_self_monitoring_service_name")]
    pub service_name: String,
    #[serde(default = "default_self_monitoring_env")]
    pub env: String,
    #[serde(default = "default_tls_ignore")]
    pub tls_ignore: bool,
}

impl Defaults {
    pub fn to_labels_hashmap(&self) -> HashMap<String, String> {
        let mut labels = HashMap::new();

        if let Some(zone) = &self.probe_zone {
            labels.insert(String::from("zone"), zone.to_string());
        }

        if let Some(probe_location) = &self.probe_location {
            labels.insert(
                String::from("latitude"),
                probe_location.latitude.to_string(),
            );
            labels.insert(
                String::from("longitude"),
                probe_location.longitude.to_string(),
            );
        }

        return labels;
    }
}

impl From<configuration::model::defaults::Defaults> for Defaults {
    fn from(value: configuration::model::defaults::Defaults) -> Self {
        Defaults {
            probe_location: value.probe_location.map(ProbeLocation::from),
            probe_zone: value.probe_zone,
            self_monitoring: value.self_monitoring.map(SelfMonitoringConfig::from),
        }
    }
}

impl From<configuration::model::defaults::ProbeLocation> for ProbeLocation {
    fn from(value: configuration::model::defaults::ProbeLocation) -> Self {
        ProbeLocation {
            latitude: value.latitude,
            longitude: value.longitude,
        }
    }
}

impl From<configuration::model::defaults::SelfMonitoringConfig> for SelfMonitoringConfig {
    fn from(value: configuration::model::defaults::SelfMonitoringConfig) -> Self {
        SelfMonitoringConfig {
            enable: value.enable,
            otel_endpoint: value.otel_endpoint,
            service_name: value.service_name,
            env: value.env,
            tls_ignore: value.tls_ignore,
        }
    }
}
