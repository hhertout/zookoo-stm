#[derive(Debug, Clone)]
pub struct ExporterConfiguration {
    pub otel: Option<OtelGrpcExporterConfiguration>,
    pub metrics: Option<MetricsExporterConfiguration>,
    pub kafka: Option<KafkaExporterConfiguration>,
}

#[derive(Debug, Clone)]
pub struct OtelGrpcExporterConfiguration {
    pub url: String,
    pub auth: Option<AuthConfiguration>,
    pub cert_path: Option<String>,
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
    pub enpoint: String,
}

#[derive(Debug, Clone)]
pub struct AuthConfiguration {
    pub user: Option<String>,
    pub password: Option<String>,
    pub bearer: Option<String>,
}

impl From<configuration::model::exporter::ExporterConfiguration> for ExporterConfiguration {
    fn from(value: configuration::model::exporter::ExporterConfiguration) -> Self {
        ExporterConfiguration {
            otel: value.otel.map(OtelGrpcExporterConfiguration::from),
            metrics: value.metrics.map(MetricsExporterConfiguration::from),
            kafka: value.kafka.map(KafkaExporterConfiguration::from),
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
        }
    }
}

impl From<configuration::model::exporter::MetricsExporterConfiguration>
    for MetricsExporterConfiguration
{
    fn from(value: configuration::model::exporter::MetricsExporterConfiguration) -> Self {
        MetricsExporterConfiguration {
            enpoint: value.enpoint,
        }
    }
}

impl From<configuration::model::exporter::AuthConfiguration> for AuthConfiguration {
    fn from(value: configuration::model::exporter::AuthConfiguration) -> Self {
        AuthConfiguration {
            user: value.user,
            password: value.password,
            bearer: value.bearer,
        }
    }
}

impl Into<exporter::config::AuthConfiguration> for AuthConfiguration {
    fn into(self) -> exporter::config::AuthConfiguration {
        exporter::config::AuthConfiguration {
            user: self.user,
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
        }
    }
}
