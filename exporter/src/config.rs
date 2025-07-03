use crate::otel::AuthHeader;

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

impl OtelGrpcExporterConfiguration {
    pub fn auth_header(&self) -> Option<AuthHeader> {
        self.auth.as_ref().and_then(|auth| {
            if let Some(bearer) = &auth.bearer {
                Some(AuthHeader::Bearer(bearer.clone()))
            } else if let (Some(user), Some(password)) = (&auth.user, &auth.password) {
                Some(AuthHeader::Basic {
                    user: user.clone(),
                    password: password.clone(),
                })
            } else {
                None
            }
        })
    }
}
