use std::env;
use std::sync::OnceLock;

use opentelemetry::global::{self, BoxedSpan, BoxedTracer};
use opentelemetry::trace::{Span, SpanKind, TraceContextExt, Tracer};
use opentelemetry::{Context, KeyValue};
use opentelemetry_otlp::{SpanExporter, WithExportConfig};
use opentelemetry_sdk::{Resource, propagation::TraceContextPropagator, trace::SdkTracerProvider};
use tokio::sync::mpsc;

use crate::config::exporter::OtelGrpcExporterConfiguration;
use crate::scrap_config::ProbeConfig;
use crate::target::http::http_scrape_with_shutdown;
use crate::target::icmp::icmp_scrape_with_shutdown;

pub(crate) mod config;
pub(crate) mod file;
pub(crate) mod group_by_interval;
pub(crate) mod metrics;
pub mod scrap_config;
pub(crate) mod target;

pub async fn run(config: ProbeConfig) {
    // Init observability stuff
    let mut tracer_provider: Option<SdkTracerProvider> = None;

    if env::var("ENABLE_SELF_MONITORING").unwrap_or(String::from("false")) == String::from("true") {
        tracer_provider = Some(init_tracer_provider());
    }

    if let Some(otel_conf) = &config.config.exporter.otel {
        let config = OtelGrpcExporterConfiguration::from(otel_conf.clone());
        let _ = exporter::otel::init_metrics_exporter(config.into());
    }

    // Run the probe
    probe_engine(config).await;

    // Shutdown obersvability stuff
    if let Some(provider) = tracer_provider {
        log::info!("Shutting down open telemetry tracer...");
        let _ = provider.shutdown();
    }
}

pub async fn probe_engine(config: ProbeConfig) {
    let (icmp_shutdown_tx, icmp_shutdown_rx) = mpsc::channel::<()>(1);
    let (http_shutdown_tx, http_shutdown_rx) = mpsc::channel::<()>(1);

    let scrape_task = tokio::spawn(icmp_scrape_with_shutdown(config.clone(), icmp_shutdown_rx));
    let _ = tokio::spawn(http_scrape_with_shutdown(config.clone(), http_shutdown_rx));

    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for Ctrl+C");

    log::info!("Ctrl+C received, shutting down...");

    let _ = icmp_shutdown_tx.send(()).await;
    let _ = http_shutdown_tx.send(()).await;
    let _ = scrape_task.await;
}

fn get_tracer() -> &'static BoxedTracer {
    static TRACER: OnceLock<BoxedTracer> = OnceLock::new();
    TRACER.get_or_init(|| global::tracer("dice_server"))
}

fn tracing_new_span(tracer: &BoxedTracer, name: String) -> BoxedSpan {
    tracer
        .span_builder(name)
        .with_kind(SpanKind::Internal)
        .start(tracer)
}

fn child_span_from_context(name: &str, ctx: Context, attr: Vec<KeyValue>) -> Context {
    let mut span = tracing_new_span_with_context(get_tracer(), name.to_string(), ctx.clone());
    span.set_attributes(attr);
    ctx.with_span(span)
}

fn tracing_new_span_with_context(
    tracer: &'static BoxedTracer,
    name: String,
    cx: Context,
) -> impl Span {
    tracer.start_with_context(name, &cx)
}

fn init_tracer_provider() -> SdkTracerProvider {
    let default_endpoint = "http://localhost:4317/v1/traces".to_string();

    let exporter = SpanExporter::builder()
        .with_tonic()
        .with_endpoint(env::var("INTERNAL_OLTP_ENDPOINT").unwrap_or(default_endpoint))
        .build()
        .expect("Failed to create span exporter");

    let provider = SdkTracerProvider::builder()
        .with_resource(
            Resource::builder()
                .with_service_name("zookoozookoo")
                .with_attribute(KeyValue::new(
                    "env",
                    env::var("RUST_ENV").unwrap_or_default(),
                ))
                .build(),
        )
        .with_batch_exporter(exporter)
        .build();

    global::set_text_map_propagator(TraceContextPropagator::new());
    global::set_tracer_provider(provider.clone());

    provider
}
