use std::collections::HashMap;

use opentelemetry::global;
use opentelemetry::trace::{Span, SpanKind, TraceContextExt, Tracer};
use opentelemetry::{Context, KeyValue};
use opentelemetry_otlp::tonic_types::transport::ClientTlsConfig;
use opentelemetry_otlp::{MetricExporter, SpanExporter, WithExportConfig, WithTonicConfig};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::trace::SdkTracerProvider;

pub type BoxedTracer = opentelemetry::global::BoxedTracer;
pub type BoxedSpan = opentelemetry::global::BoxedSpan;

/// Initialize the OpenTelemetry tracer provider with the given configuration.
/// This function sets up the OpenTelemetry tracer provider with the specified endpoint, service name, environment, and zone.
/// It configures the span exporter to use gRPC with the provided endpoint and TLS configuration if applicable.
/// The tracer provider is then built and set as the global tracer provider, allowing for tracing across the application.
/// The function returns the initialized tracer provider for further use.
pub fn init_tracer_provider(
    endpoint: String,
    service_name: String,
    env: String,
    zone: Option<String>,
) -> SdkTracerProvider {
    let mut builder =
        SpanExporter::builder().with_tonic().with_endpoint(format!("{}/v1/traces", endpoint));

    if endpoint.starts_with("https") {
        builder = builder.with_tls_config(ClientTlsConfig::new().with_enabled_roots())
    }

    let exporter = builder.build().expect("Failed to create span exporter");

    let provider = SdkTracerProvider::builder()
        .with_resource(
            Resource::builder()
                .with_service_name(service_name)
                .with_attribute(KeyValue::new("env", env))
                .with_attribute(KeyValue::new("zone", zone.unwrap_or("world_wide".to_string())))
                .build(),
        )
        .with_batch_exporter(exporter)
        .build();

    global::set_tracer_provider(provider.clone());

    provider
}

/// Initialize the OpenTelemetry meter provider for metrics export.
/// This function sets up the meter provider with OTLP gRPC export to the specified endpoint.
pub fn init_meter_provider(
    endpoint: String,
    service_name: String,
    env: String,
    zone: Option<String>,
) -> SdkMeterProvider {
    use opentelemetry_sdk::metrics::PeriodicReader;
    use std::time::Duration;

    let mut builder =
        MetricExporter::builder().with_tonic().with_endpoint(format!("{}/v1/metrics", endpoint));

    if endpoint.starts_with("https") {
        builder = builder.with_tls_config(ClientTlsConfig::new().with_enabled_roots())
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

    global::set_meter_provider(provider.clone());

    log::info!("event=meter_provider_initialized endpoint={} interval_s=15", endpoint);

    provider
}

/// Get the tracer instance
/// This function initializes the tracer if it is not already initialized and returns a reference to it.
/// The tracer is used to create spans for tracing the execution of the application.
pub fn get_tracer() -> &'static BoxedTracer {
    use std::sync::OnceLock;
    static TRACER: OnceLock<BoxedTracer> = OnceLock::new();
    TRACER.get_or_init(|| global::tracer("zookoo"))
}

/// Create a new span with the given name using the global tracer.
/// This function creates a new span with the specified name and starts it using the global tracer.
/// The span is created with the `SpanKind::Internal` kind, indicating that it is an internal operation within the application.
pub fn tracing_new_span(tracer: &BoxedTracer, name: String) -> BoxedSpan {
    tracer.span_builder(name).with_kind(SpanKind::Internal).start(tracer)
}

/// Create a new child span from the given context with the specified name and attributes.
/// This function creates a new child span from the provided context, allowing for tracing of operations that are related to the parent span.
/// The new span is created with the specified name and attributes, and it inherits the context of the parent span.
/// The span is then returned as a new context that can be used for further tracing operations.
pub fn child_span_from_context(name: &str, ctx: Context, attr: Vec<KeyValue>) -> Context {
    let mut span = tracing_new_span_with_context(get_tracer(), name.to_string(), ctx.clone());
    span.set_attributes(attr);
    TraceContextExt::with_span(&ctx, span)
}

/// Create a new span with the given name and context using the global tracer.
/// This function creates a new span with the specified name and associates it with the provided context.
/// The span is created using the global tracer, allowing for tracing of operations that are related to the context.
/// The span is returned as an implementation of the `Span` trait, which can be used for further tracing operations.
#[doc(hidden)]
pub fn tracing_new_span_with_context(
    tracer: &'static BoxedTracer,
    name: String,
    cx: Context,
) -> impl Span {
    tracer.start_with_context(name, &cx)
}

pub fn get_empty_attributes() -> HashMap<String, String> {
    HashMap::new()
}

/// Create a span with name and attributes
///
/// Usage:
/// ```ignore
/// use probe::span;
/// use std::collections::HashMap;
///
/// let mut attrs = HashMap::new();
/// attrs.insert("key".to_string(), "value".to_string());
/// let ctx = span!("my_span".to_string(), attrs);
/// ```
#[macro_export]
macro_rules! span {
    ($name:expr, $attributes:expr) => {{
        let mut span =
            $crate::observability::tracing_new_span($crate::observability::get_tracer(), $name);
        for (key, value) in $attributes.iter() {
            span.set_attribute(opentelemetry::KeyValue::new(key.clone(), value.clone()));
        }
        opentelemetry::Context::current_with_span(span)
    }};
}

/// Create a child span with context, name and attributes
///
/// Usage:
/// ```ignore
/// use probe::child_span;
/// use std::collections::HashMap;
///
/// let ctx = opentelemetry::Context::current();
/// let mut attrs = HashMap::new();
/// attrs.insert("key".to_string(), "value".to_string());
/// let new_ctx = child_span!(ctx, "child".to_string(), attrs);
/// ```
#[macro_export]
macro_rules! child_span {
    ($ctx:expr, $name:expr, $attributes:expr) => {{
        let mut span = $crate::observability::tracing_new_span_with_context(
            $crate::observability::get_tracer(),
            $name,
            $ctx.clone(),
        );
        for (key, value) in $attributes.iter() {
            span.set_attribute(opentelemetry::KeyValue::new(key.clone(), value.clone()));
        }
        opentelemetry::trace::TraceContextExt::with_span(&$ctx, span)
    }};
}
