use std::{process::exit, sync::OnceLock};

use base64::Engine;
use base64::engine::general_purpose;
use opentelemetry::global::{self};
use opentelemetry_otlp::{
    MetricExporter, WithExportConfig, WithTonicConfig,
    tonic_types::{
        metadata::MetadataMap,
        transport::{Certificate, ClientTlsConfig},
    },
};
use opentelemetry_sdk::{Resource, metrics::SdkMeterProvider};

use crate::config::OtelGrpcExporterConfiguration;
use std::fs;

pub mod metrics;

pub enum AuthHeader {
    Bearer(String),
    Basic { user: String, password: String },
}

impl AuthHeader {
    pub fn to_metadata(&self) -> MetadataMap {
        let mut metadata = MetadataMap::new();
        let header_value = match self {
            AuthHeader::Bearer(token) => format!("Bearer {}", token),
            AuthHeader::Basic { user, password } => {
                let credentials = format!("{}:{}", user, password);
                let encoded = general_purpose::STANDARD.encode(credentials);
                format!("Basic {}", encoded)
            }
        };
        metadata.insert("authorization", header_value.parse().unwrap());
        metadata
    }
}

fn get_resource() -> Resource {
    static RESOURCE: OnceLock<Resource> = OnceLock::new();
    RESOURCE
        .get_or_init(|| Resource::builder().with_service_name("rustbox").build())
        .clone()
}

pub fn init_metrics_exporter(config: OtelGrpcExporterConfiguration) -> SdkMeterProvider {
    log::warn!(
        "sending otel metrics with grpc to '{}' endpoint",
        config.url
    );

    let mut builder = MetricExporter::builder()
        .with_tonic()
        .with_endpoint(config.url.clone());

    if let Some(auth) = config.auth_header() {
        log::warn!("otel authentication enable");
        builder = builder.with_metadata(auth.to_metadata());
    }

    if let Some(cert_path) = config.cert_path {
        log::warn!("otel custom certificate enable");
        if let Ok(pem) = fs::read_to_string(&cert_path) {
            let ca_certificate = Certificate::from_pem(pem);
            let tls_config = ClientTlsConfig::new().ca_certificate(ca_certificate);

            builder = builder.with_tls_config(tls_config)
        } else {
            log::error!("FAIL TO GET CUSTOM CERT FILE FROM PATH {}", cert_path);
            exit(1);
        };
    }

    let exporter = builder.build().expect("fail to create metric exporter");

    let meter_provider = SdkMeterProvider::builder()
        .with_periodic_exporter(exporter)
        .with_resource(get_resource())
        .build();

    global::set_meter_provider(meter_provider.clone());
    meter_provider
}

pub fn shutdown(meter_exporter: SdkMeterProvider) -> Result<(), String> {
    let mut shutdown_errors = Vec::new();
    if let Err(e) = meter_exporter.shutdown() {
        shutdown_errors.push(format!("meter provider: {e}"));
    }

    if !shutdown_errors.is_empty() {
        return Err(format!(
            "Failed to shutdown providers:{}",
            shutdown_errors.join("\n")
        )
        .into());
    }

    Ok(())
}
