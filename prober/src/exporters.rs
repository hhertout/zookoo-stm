use opentelemetry::global;
use opentelemetry::{KeyValue};
use opentelemetry_otlp::{MetricExporter, WithExportConfig, WithTonicConfig};
use opentelemetry_otlp::tonic_types::transport::ClientTlsConfig;
use opentelemetry_sdk::metrics::PeriodicReader;
use opentelemetry_sdk::Resource;
use std::time::Duration;

/// Initialize the OpenTelemetry metrics exporter with the given configuration.
/// This function sets up the OpenTelemetry metrics exporter with the specified endpoint, export interval, and resource attributes.
/// It configures the metric exporter to use gRPC with the provided endpoint and TLS configuration if applicable.
/// The periodic reader is then built with the specified export interval and set as the global meter provider, allowing for metrics collection across the application.
/// 
/// Note: This function is currently not used but will be integrated when OpenTelemetry metrics export is enabled.
#[allow(dead_code)]
pub fn init_opentelemetry_metrics(
    endpoint: String,
    service_name: String,
    env: String,
    zone: Option<String>,
    export_interval: Duration,
) {
    let mut builder = MetricExporter::builder()
        .with_tonic()
        .with_endpoint(format!("{}/v1/metrics", endpoint));

    if endpoint.starts_with("https") {
        builder = builder.with_tls_config(ClientTlsConfig::new().with_native_roots())
    }

    let exporter = builder
        .build()
        .expect("Failed to create OTEL metrics exporter");

    let reader = PeriodicReader::builder(exporter)
        .with_interval(export_interval)
        .build();

    let provider = opentelemetry_sdk::metrics::SdkMeterProvider::builder()
        .with_resource(
            Resource::builder()
                .with_service_name(service_name)
                .with_attribute(KeyValue::new("env", env))
                .with_attribute(KeyValue::new(
                    "zone",
                    zone.unwrap_or("world_wide".to_string()),
                ))
                .build(),
        )
        .with_reader(reader)
        .build();

    global::set_meter_provider(provider);
}

// Prometheus Pushgateway support has been removed
// Use prometheus_remote_write instead for exporting to Prometheus-compatible endpoints
