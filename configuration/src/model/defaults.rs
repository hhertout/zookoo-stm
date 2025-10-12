use serde::Deserialize;

fn default_log_level() -> String {
    return String::from("info");
}

fn default_tls_ignore() -> bool {
    return false;
}

fn default_self_monitoring_enabled() -> bool {
    return false;
}

fn default_self_monitoring_otel_endpoint() -> String {
    return String::from("http://localhost:4317");
}

fn default_self_monitoring_pyroscope_endpoint() -> String {
    return String::from("http://localhost:9999");
}

fn default_self_monitoring_service_name() -> String {
    return String::from("zookoo");
}

fn default_self_monitoring_env() -> String {
    return String::from("development");
}

#[derive(Debug, Clone, Deserialize)]
pub struct Defaults {
    #[serde(default = "default_log_level")]
    pub log_level: String,
    pub probe_location: Option<ProbeLocation>,
    pub probe_zone: Option<String>,
    pub self_monitoring: SelfMonitoringConfig,
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
    #[serde(default = "default_self_monitoring_pyroscope_endpoint")]
    pub pyroscope_endpoint: String,
    #[serde(default = "default_self_monitoring_service_name")]
    pub service_name: String,
    #[serde(default = "default_self_monitoring_env")]
    pub env: String,
    #[serde(default = "default_tls_ignore")]
    pub tls_ignore: bool,
}
