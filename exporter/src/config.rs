use crate::otel::AuthHeader;

#[derive(Debug, Clone)]
pub struct ExporterConfiguration {
    pub otel: Option<OtelGrpcExporterConfiguration>,
    pub metrics: Option<MetricsExporterConfiguration>,
    pub kafka: Option<KafkaExporterConfiguration>,
    pub prometheus: Option<PrometheusPushgatewayConfiguration>,
}

#[derive(Debug, Clone)]
pub struct PrometheusPushgatewayConfiguration {
    pub url: String,
    pub job: String,
    pub instance: Option<String>,
    pub auth: Option<AuthConfiguration>,
}

#[derive(Debug, Clone)]
pub struct OtelGrpcExporterConfiguration {
    pub url: String,
    pub auth: Option<AuthConfiguration>,
    pub tls_insecure: bool,
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
    pub endpoint: String,
}

#[derive(Debug, Clone)]
pub struct AuthConfiguration {
    pub username: Option<String>,
    pub password: Option<String>,
    pub bearer: Option<String>,
}

impl OtelGrpcExporterConfiguration {
    pub fn auth_header(&self) -> Option<AuthHeader> {
        self.auth.as_ref().and_then(|auth| {
            if let Some(bearer) = &auth.bearer {
                Some(AuthHeader::Bearer(bearer.clone()))
            } else if let (Some(username), Some(password)) = (&auth.username, &auth.password) {
                Some(AuthHeader::Basic { username: username.clone(), password: password.clone() })
            } else {
                None
            }
        })
    }
}
