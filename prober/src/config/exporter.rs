///! This module defines the configuration for exporters in the prober application.
///! It includes the structure for exporter configurations, such as OpenTelemetry, Kafka, and Metrics exporters.
///! The configurations are used to define how the data should be exported, including endpoints, authentication

#[derive(Debug, Clone)]
pub struct ExporterConfiguration {
    pub otel: Option<OtelGrpcExporterConfiguration>,
    pub metrics: Option<MetricsExporterConfiguration>,
    pub kafka: Option<KafkaExporterConfiguration>,
    pub prometheus_remote_write: Option<PrometheusRemoteWriteConfiguration>,
    pub timescale: Option<TimescaleExporterConfiguration>,
}

#[derive(Debug, Clone)]
pub struct PrometheusRemoteWriteConfiguration {
    pub url: String,
    pub job: String,
    pub instance: Option<String>,
    pub auth: Option<AuthConfiguration>,
}

#[derive(Debug, Clone)]
pub struct TimescaleExporterConfiguration {
    pub connection_string: String,
    pub schema: String,
}

#[derive(Debug, Clone)]
pub struct OtelGrpcExporterConfiguration {
    pub url: String,
    pub auth: Option<AuthConfiguration>,
    pub cert_path: Option<String>,
    pub tls_insecure: bool,
}

#[derive(Debug, Clone)]
pub struct KafkaExporterConfiguration {
    pub broker: String,
    pub topic: String,
    pub auth: Option<AuthConfiguration>,
    pub cert_path: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MetricsExporterConfiguration {
    pub endpoint: String,
}

#[derive(Debug, Clone)]
pub struct AuthConfiguration {
    pub username: Option<String>,
    pub password: Option<String>,
    pub bearer: Option<String>,
}

impl From<configuration::model::exporter::ExporterConfiguration> for ExporterConfiguration {
    fn from(value: configuration::model::exporter::ExporterConfiguration) -> Self {
        ExporterConfiguration {
            otel: value.otel.map(OtelGrpcExporterConfiguration::from),
            metrics: value.metrics.map(MetricsExporterConfiguration::from),
            kafka: value.kafka.map(KafkaExporterConfiguration::from),
            prometheus_remote_write: value.prometheus_remote_write.map(PrometheusRemoteWriteConfiguration::from),
            timescale: value.timescale.map(TimescaleExporterConfiguration::from),
        }
    }
}

impl From<configuration::model::exporter::KafkaExporterConfiguration>
    for KafkaExporterConfiguration
{
    fn from(value: configuration::model::exporter::KafkaExporterConfiguration) -> Self {
        KafkaExporterConfiguration {
            broker: value.broker,
            topic: value.topic,
            auth: value.auth.map(AuthConfiguration::from),
            cert_path: value.cert_path,
        }
    }
}

impl From<configuration::model::exporter::OtelGrpcExporterConfiguration>
    for OtelGrpcExporterConfiguration
{
    fn from(value: configuration::model::exporter::OtelGrpcExporterConfiguration) -> Self {
        OtelGrpcExporterConfiguration {
            url: value.url,
            auth: value.auth.map(AuthConfiguration::from),
            cert_path: value.cert_path,
            tls_insecure: value.tls_insecure,
        }
    }
}

impl From<configuration::model::exporter::MetricsExporterConfiguration>
    for MetricsExporterConfiguration
{
    fn from(value: configuration::model::exporter::MetricsExporterConfiguration) -> Self {
        MetricsExporterConfiguration {
            endpoint: value.endpoint,
        }
    }
}

impl From<configuration::model::exporter::AuthConfiguration> for AuthConfiguration {
    fn from(value: configuration::model::exporter::AuthConfiguration) -> Self {
        AuthConfiguration {
            username: value.username,
            password: value.password,
            bearer: value.bearer,
        }
    }
}

impl From<configuration::model::exporter::PrometheusRemoteWriteConfiguration>
    for PrometheusRemoteWriteConfiguration
{
    fn from(value: configuration::model::exporter::PrometheusRemoteWriteConfiguration) -> Self {
        PrometheusRemoteWriteConfiguration {
            url: value.url,
            job: value.job,
            instance: value.instance,
            auth: value.auth.map(AuthConfiguration::from),
        }
    }
}

impl From<configuration::model::exporter::TimescaleExporterConfiguration>
    for TimescaleExporterConfiguration
{
    fn from(value: configuration::model::exporter::TimescaleExporterConfiguration) -> Self {
        TimescaleExporterConfiguration {
            connection_string: value.connection_string,
            schema: value.schema,
        }
    }
}

impl Into<exporter::config::AuthConfiguration> for AuthConfiguration {
    fn into(self) -> exporter::config::AuthConfiguration {
        exporter::config::AuthConfiguration {
            username: self.username,
            password: self.password,
            bearer: self.bearer,
        }
    }
}

impl Into<exporter::config::OtelGrpcExporterConfiguration> for OtelGrpcExporterConfiguration {
    fn into(self) -> exporter::config::OtelGrpcExporterConfiguration {
        exporter::config::OtelGrpcExporterConfiguration {
            url: self.url,
            auth: self.auth.map(Into::into),
            cert_path: self.cert_path,
            tls_insecure: self.tls_insecure,
        }
    }
}
