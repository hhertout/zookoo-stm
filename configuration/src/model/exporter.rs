use serde::Deserialize;

fn default_tls_insecure() -> bool {
    return false;
}

fn default_prometheus_job() -> String {
    "zookoo-stm".to_string()
}

#[derive(Debug, Deserialize)]
pub struct ExporterConfiguration {
    pub otel: Option<OtelGrpcExporterConfiguration>,
    pub metrics: Option<MetricsExporterConfiguration>,
    pub kafka: Option<KafkaExporterConfiguration>,
    pub prometheus_remote_write: Option<PrometheusRemoteWriteConfiguration>,
}

#[derive(Debug, Deserialize)]
pub struct OtelGrpcExporterConfiguration {
    pub url: String,
    pub auth: Option<AuthConfiguration>,
    pub cert_path: Option<String>,
    #[serde(default = "default_tls_insecure")]
    pub tls_insecure: bool,
}

#[derive(Debug, Deserialize)]
pub struct KafkaExporterConfiguration {
    pub broker: String,
    pub topic: String,
    pub auth: Option<AuthConfiguration>,
    pub cert_path: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MetricsExporterConfiguration {
    pub enpoint: String,
}

#[derive(Debug, Deserialize)]
pub struct AuthConfiguration {
    pub username: Option<String>,
    pub password: Option<String>,
    pub bearer: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PrometheusRemoteWriteConfiguration {
    pub url: String,
    #[serde(default = "default_prometheus_job")]
    pub job: String,
    pub instance: Option<String>,
    pub auth: Option<AuthConfiguration>,
}
