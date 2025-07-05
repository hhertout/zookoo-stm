use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ExporterConfiguration {
    pub otel: Option<OtelGrpcExporterConfiguration>,
    pub metrics: Option<MetricsExporterConfiguration>,
    pub kafka: Option<KafkaExporterConfiguration>,
}

#[derive(Debug, Deserialize)]
pub struct OtelGrpcExporterConfiguration {
    pub url: String,
    pub auth: Option<AuthConfiguration>,
    pub cert_path: Option<String>,
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
