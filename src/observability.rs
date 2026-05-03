use base64::Engine;
use configuration::model::defaults::SelfMonitoringConfig;
use opentelemetry::trace::TracerProvider;
use opentelemetry::{KeyValue, global};
use opentelemetry_appender_tracing::layer;
use opentelemetry_otlp::{
    SpanExporter, WithExportConfig, WithTonicConfig,
    tonic_types::{metadata::MetadataMap, transport::ClientTlsConfig},
};
use opentelemetry_sdk::{Resource, logs::SdkLoggerProvider, trace::SdkTracerProvider};
use pyroscope::backend::{BackendConfig, PprofConfig, pprof_backend};
use pyroscope::pyroscope::{PyroscopeAgentBuilder, PyroscopeAgentReady};
use pyroscope::PyroscopeAgent;
use tracing_subscriber::{EnvFilter, fmt};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn start_pyroscope(
    config: SelfMonitoringConfig,
) -> Result<PyroscopeAgent<PyroscopeAgentReady>, Box<dyn std::error::Error>> {
    let backend_config =
        BackendConfig { report_thread_id: true, report_thread_name: true, report_pid: false };
    let backend_impl = pprof_backend(PprofConfig::default(), backend_config);
    let hostname = hostname::get().unwrap_or_default().to_string_lossy().to_string();

    let mut builder = PyroscopeAgentBuilder::new(
        config.pyroscope_endpoint,
        config.service_name,
        100,
        "pyroscope-rs",
        env!("CARGO_PKG_VERSION"),
        backend_impl,
    )
    .tags(vec![("host", &hostname)]);

    if let Some(basic_auth) = config.basic_auth {
        builder = builder.basic_auth(basic_auth.username, basic_auth.password);
    }

    if let Some(bearer) = config.bearer {
        let mut headers = std::collections::HashMap::new();
        headers.insert("Authorization".to_string(), format!("Bearer {}", bearer));
        builder = builder.http_headers(headers);
    }

    let agent = builder.build()?;
    Ok(agent)
}

pub struct ObservabilityGuard {
    tracer_provider: SdkTracerProvider,
    logger_provider: SdkLoggerProvider,
}

impl Drop for ObservabilityGuard {
    fn drop(&mut self) {
        let _ = self.tracer_provider.shutdown();
        let _ = self.logger_provider.shutdown();
    }
}

pub fn init_observability(
    log_level: &str,
    self_monitoring_config: SelfMonitoringConfig,
) -> ObservabilityGuard {
    let log_level_to_apply = match log_level {
        "error" | "warn" | "debug" | "info" | "trace" => log_level,
        _ => "info",
    };

    // Derive OTEL log level from app log level
    let otel_level = match log_level_to_apply {
        "debug" | "trace" => "debug",
        _ => "warn",
    };

    // Build filter
    let filter = format!(
        "{},opentelemetry={},opentelemetry_sdk={},opentelemetry_otlp={},h2=warn,hyper=warn,tower=warn,tonic=warn",
        log_level_to_apply, otel_level, otel_level, otel_level
    );

    let registry = tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&filter)))
        .with(fmt::layer().with_target(true).with_thread_names(true));

    // Initialize OTEL providers if self-monitoring is enabled

    let tp = init_tracer_provider(self_monitoring_config.clone());
    let lp = init_logger_provider(self_monitoring_config);

    // Add OTEL layers
    let tracer = tp.tracer("zookoo");
    let otel_trace_layer = tracing_opentelemetry::layer().with_tracer(tracer);
    let otel_log_layer = layer::OpenTelemetryTracingBridge::new(&lp);

    registry.with(otel_trace_layer).with(otel_log_layer).init();

    log::warn!("event=self_monitoring_enabled");

    ObservabilityGuard { tracer_provider: tp, logger_provider: lp }
}

/// Initialize tracer provider without setting up the subscriber
fn init_tracer_provider(config: SelfMonitoringConfig) -> SdkTracerProvider {
    let endpoint = config.otel_endpoint.clone();
    let mut builder = SpanExporter::builder().with_tonic().with_endpoint(endpoint.clone());

    if endpoint.starts_with("https") {
        builder = builder.with_tls_config(ClientTlsConfig::new().with_enabled_roots())
    }

    let mut metadata = MetadataMap::new();
    if let Some(basic_auth) = config.basic_auth.as_ref() {
        let credentials = base64::engine::general_purpose::STANDARD
            .encode(format!("{}:{}", basic_auth.username, basic_auth.password));
        metadata.insert("authorization", format!("Basic {}", credentials).parse().unwrap());
    }
    if let Some(bearer) = config.bearer.as_ref() {
        metadata.insert("authorization", format!("Bearer {}", bearer).parse().unwrap());
    }
    if !metadata.is_empty() {
        builder = builder.with_metadata(metadata);
    }

    let exporter = builder.build().expect("Failed to create span exporter");

    let provider = SdkTracerProvider::builder()
        .with_resource(
            Resource::builder()
                .with_service_name(config.service_name)
                .with_attribute(KeyValue::new("env", config.env))
                .with_attribute(KeyValue::new(
                    "zone",
                    config.zone.unwrap_or("world_wide".to_string()),
                ))
                .build(),
        )
        .with_batch_exporter(exporter)
        .build();

    global::set_tracer_provider(provider.clone());
    provider
}

/// Initialize logger provider without setting up the subscriber
fn init_logger_provider(config: SelfMonitoringConfig) -> SdkLoggerProvider {
    let exporter = opentelemetry_otlp::LogExporter::builder()
        .with_tonic()
        .with_endpoint(config.otel_endpoint)
        .build()
        .expect("Failed to create OTLP log exporter");

    SdkLoggerProvider::builder()
        .with_resource(Resource::builder().with_service_name(config.service_name).build())
        .with_batch_exporter(exporter)
        .build()
}
