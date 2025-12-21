use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use base64::Engine;

use configuration::model::exporter::{AuthConfiguration, OtelGrpcExporterConfiguration};
use opentelemetry::KeyValue;
use opentelemetry_otlp::tonic_types::{
    metadata::MetadataMap,
    transport::{Certificate, ClientTlsConfig},
};
use opentelemetry_otlp::{MetricExporter, WithExportConfig, WithTonicConfig};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use std::fs;
use tokio::time::sleep;

pub type BoxedTracer = opentelemetry::global::BoxedTracer;
pub type BoxedSpan = opentelemetry::global::BoxedSpan;

fn build_auth_metadata(auth: &Option<AuthConfiguration>) -> MetadataMap {
    let mut metadata = MetadataMap::new();

    if let Some(auth) = auth.as_ref() {
        if let (Some(username), Some(password)) = (auth.username.as_ref(), auth.password.as_ref()) {
            let credentials = base64::engine::general_purpose::STANDARD
                .encode(format!("{}:{}", username, password));
            metadata.insert("authorization", format!("Basic {}", credentials).parse().unwrap());
        }
        if let Some(bearer) = auth.bearer.as_ref() {
            // Bearer wins if both are present.
            metadata.insert("authorization", format!("Bearer {}", bearer).parse().unwrap());
        }
    }

    metadata
}

/// Extract host and port from an endpoint URL (e.g., "http://localhost:4317" -> ("localhost", 4317))
fn parse_otel_endpoint(url: &str) -> Option<(String, u16)> {
    let without_scheme =
        url.strip_prefix("https://").or_else(|| url.strip_prefix("http://")).unwrap_or(url);

    let parts: Vec<&str> = without_scheme.split('/').next()?.split(':').collect();

    match parts.len() {
        1 => Some((parts[0].to_string(), 4317)), // Default gRPC port
        2 => {
            let port = parts[1].parse().ok()?;
            Some((parts[0].to_string(), port))
        }
        _ => None,
    }
}

/// Check if the OTEL endpoint is reachable via TCP
fn check_otel_endpoint_reachable(host: &str, port: u16) -> bool {
    let addr = format!("{}:{}", host, port);
    let timeout = Duration::from_secs(3);

    // Try to resolve and connect
    if let Ok(addrs) = addr.to_socket_addrs() {
        for socket_addr in addrs {
            if TcpStream::connect_timeout(&socket_addr, timeout).is_ok() {
                return true;
            }
        }
    }
    false
}

/// Initialize the OpenTelemetry meter provider for metrics export.
/// This function sets up the meter provider with OTLP gRPC export to the specified endpoint.
pub fn init_meter_provider(
    config: OtelGrpcExporterConfiguration,
    service_name: String,
    env: String,
    zone: Option<String>,
) -> SdkMeterProvider {
    use opentelemetry_sdk::metrics::PeriodicReader;

    let endpoint = config.url.clone();

    // Health check: verify endpoint is reachable
    if let Some((host, port)) = parse_otel_endpoint(&endpoint) {
        if check_otel_endpoint_reachable(&host, port) {
            log::info!("event=otel_endpoint_reachable host={} port={}", host, port);
        } else {
            let _host = host.clone();
            tokio::spawn(async move {
                loop {
                    log::error!(
                        "event=otel_endpoint_unreachable host={} port={} msg=OTEL endpoint is not reachable. Exporting failed!",
                        _host,
                        port
                    );

                    sleep(Duration::from_secs(10)).await;
                }
            });
        }
    } else {
        log::warn!("event=otel_endpoint_parse_failed url={}", endpoint);
    }

    let mut builder = MetricExporter::builder().with_tonic().with_endpoint(endpoint.clone());

    if endpoint.starts_with("https") {
        let mut tls_config = ClientTlsConfig::new();
        if config.tls_insecure {
            tls_config = tls_config.with_enabled_roots();
        } else {
            tls_config = tls_config.with_native_roots();
        }
        builder = builder.with_tls_config(tls_config)
    }

    // Auth metadata (bearer/basic)
    let metadata = build_auth_metadata(&config.auth);
    if !metadata.is_empty() {
        builder = builder.with_metadata(metadata);
    }

    // Optional custom CA bundle
    if let Some(cert_path) = config.cert_path.as_ref() {
        if let Ok(pem) = fs::read_to_string(cert_path) {
            let ca_certificate = Certificate::from_pem(pem);
            let tls_config = ClientTlsConfig::new().ca_certificate(ca_certificate);
            builder = builder.with_tls_config(tls_config)
        } else {
            log::error!("event=error msg=failed_to_read_custom_ca cert_path={}", cert_path);
        }
    }

    let exporter = builder.build().expect("Failed to create metric exporter");

    // Create periodic reader with custom interval (15s instead of default 60s)
    let reader = PeriodicReader::builder(exporter).with_interval(Duration::from_secs(15)).build();

    let provider = SdkMeterProvider::builder()
        .with_resource(
            Resource::builder()
                .with_service_name(service_name)
                .with_attribute(KeyValue::new("env", env))
                .with_attribute(KeyValue::new("zone", zone.unwrap_or("world_wide".to_string())))
                .build(),
        )
        .with_reader(reader)
        .build();

    log::info!("event=meter_provider_initialized endpoint={} interval_s=15", endpoint);

    provider
}
