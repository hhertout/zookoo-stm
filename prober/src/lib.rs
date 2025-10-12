//! # Prober crate
//!
//! This crate is responsible of the scraping process of the different targets define in the configuration file
//!
//! ## Behavior
//!
//! Powered by the `probe_engine` function
//!  
//! for each interval:
//!
//! - launch one job per interval
//! - each interval launch one job per target
//! - each target job complete send metrics in the same job
//!
//! ## Usage
//!
//! To start a scraping session, simply run
//!
//! ```rust
//! prober::run(ProbeConfig::from(config)).await;
//! ```
//!
//! The configuration should be complient with the `configuration` crate already present in the repository.

use opentelemetry::global::{self, BoxedSpan, BoxedTracer};
use opentelemetry::trace::{Span, SpanKind, TraceContextExt, Tracer};
use opentelemetry::{Context, KeyValue};
use opentelemetry_otlp::tonic_types::transport::ClientTlsConfig;
use opentelemetry_otlp::{SpanExporter, WithExportConfig, WithTonicConfig};
use opentelemetry_sdk::{Resource, propagation::TraceContextPropagator, trace::SdkTracerProvider};
use std::sync::OnceLock;
use tokio::sync::mpsc;

use crate::config::exporter::OtelGrpcExporterConfiguration;
use crate::config::target::{HttpTarget, IcmpTarget};
use crate::scrap_config::ProbeConfig;
use crate::target::http::scrape::HttpScrapper;
use crate::target::icmp::scrape::IcmpScrapper;
use crate::target::scrape_with_shutdown;

pub(crate) mod config;
pub(crate) mod group_by_interval;
pub(crate) mod metrics;
pub mod scrap_config;
pub(crate) mod target;

pub async fn run(config: ProbeConfig) {
    // Init observability stuff
    let mut tracer_provider: Option<SdkTracerProvider> = None;

    if config.scrap_config.default.self_monitoring.enable {
        let zone = config.scrap_config.default.probe_zone.clone();
        let self_monitoring_conf = config.scrap_config.default.self_monitoring.clone();
        tracer_provider = Some(init_tracer_provider(
            self_monitoring_conf.otel_endpoint.clone(),
            self_monitoring_conf.service_name.clone(),
            self_monitoring_conf.env.clone(),
            zone,
        ));
    }

    if let Some(otel_conf) = &config.scrap_config.exporter.otel {
        let config = OtelGrpcExporterConfiguration::from(otel_conf.clone());
        let _ = exporter::otel::init_metrics_exporter(config.into());
        log::warn!("otel exporter is enabled");
    }

    //
    // Run the probe and start the scraping
    //
    launch_probe_engine(config).await;

    // Shutdown obersvability stuff
    if let Some(provider) = tracer_provider {
        log::info!("Shutting down open telemetry tracer...");
        let _ = provider.shutdown();
    }
}

/// Probe engine definition
/// This is the starting point where the application will launch each scraping job depending on the configuration provided.
///
/// For instance... all the jobs are defined by the scraping interval at the root.
/// This means that, for each group interval, one thread will be created.
///
/// For each of this threads created, an other one is created for each scraping target will be created.
/// It ensure each scraping job is independant from each other.
///
/// The behavior of the scraping method and metric creation is defined on the dedicated module, refering to the target specification.
///
pub async fn launch_probe_engine(mut config: ProbeConfig) {
    let (icmp_shutdown_tx, icmp_shutdown_rx) = mpsc::channel::<()>(1);
    let (http_shutdown_tx, http_shutdown_rx) = mpsc::channel::<()>(1);

    // group by interval to spawn one job for each
    let icmp_group_by = config.apply_default_labels().icmp_group_by_interval();
    let http_group_by = config.apply_default_labels().http_group_by_interval();

    // launch the scraping process
    // for each interval -
    // |_ launch 1 job per interval
    //    |_ each interval launch one job per target
    //    |_ each target job complete send metrics in the same job
    //
    let scrape_task = tokio::spawn(scrape_with_shutdown::<HttpScrapper, HttpTarget>(
        http_group_by,
        http_shutdown_rx,
    ));
    let _ = tokio::spawn(scrape_with_shutdown::<IcmpScrapper, IcmpTarget>(
        icmp_group_by,
        icmp_shutdown_rx,
    ));

    // waiting for the process stop
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for Ctrl+C");

    log::info!("Ctrl+C received, shutting down...");

    // clean close of the jobs
    let _ = icmp_shutdown_tx.send(()).await;
    let _ = http_shutdown_tx.send(()).await;
    let _ = scrape_task.await;
}

/// Get the tracer instance
/// This function initializes the tracer if it is not already initialized and returns a reference to it.
/// The tracer is used to create spans for tracing the execution of the application.
fn get_tracer() -> &'static BoxedTracer {
    static TRACER: OnceLock<BoxedTracer> = OnceLock::new();
    TRACER.get_or_init(|| global::tracer("zookoo"))
}

/// Create a new span with the given name using the global tracer.
/// This function creates a new span with the specified name and starts it using the global tracer.
/// The span is created with the `SpanKind::Internal` kind, indicating that it is an internal operation within the application.
fn tracing_new_span(tracer: &BoxedTracer, name: String) -> BoxedSpan {
    tracer
        .span_builder(name)
        .with_kind(SpanKind::Internal)
        .start(tracer)
}

/// Create a new child span from the given context with the specified name and attributes.
/// This function creates a new child span from the provided context, allowing for tracing of operations that are related to the parent span.
/// The new span is created with the specified name and attributes, and it inherits the context of the parent span.
/// The span is then returned as a new context that can be used for further tracing operations.
fn child_span_from_context(name: &str, ctx: Context, attr: Vec<KeyValue>) -> Context {
    let mut span = tracing_new_span_with_context(get_tracer(), name.to_string(), ctx.clone());
    span.set_attributes(attr);
    ctx.with_span(span)
}

/// Create a new span with the given name and context using the global tracer.
/// This function creates a new span with the specified name and associates it with the provided context.
/// The span is created using the global tracer, allowing for tracing of operations that are related to the context.
/// The span is returned as an implementation of the `Span` trait, which can be used for further tracing operations.
fn tracing_new_span_with_context(
    tracer: &'static BoxedTracer,
    name: String,
    cx: Context,
) -> impl Span {
    tracer.start_with_context(name, &cx)
}

/// Initialize the OpenTelemetry tracer provider with the given configuration.
/// This function sets up the OpenTelemetry tracer provider with the specified endpoint, service name, environment, and zone.
/// It configures the span exporter to use gRPC with the provided endpoint and TLS configuration if applicable.
/// The tracer provider is then built and set as the global tracer provider, allowing for tracing across the application.
/// The function returns the initialized tracer provider for further use.
fn init_tracer_provider(
    endpoint: String,
    service_name: String,
    env: String,
    zone: Option<String>,
) -> SdkTracerProvider {
    let mut builder = SpanExporter::builder()
        .with_tonic()
        .with_endpoint(format!("{}/v1/traces", endpoint));

    if endpoint.starts_with("https") {
        builder = builder.with_tls_config(ClientTlsConfig::new().with_native_roots())
    }

    let exporter = builder.build().expect("Failed to create span exporter");

    let provider = SdkTracerProvider::builder()
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
        .with_batch_exporter(exporter)
        .build();

    global::set_text_map_propagator(TraceContextPropagator::new());
    global::set_tracer_provider(provider.clone());

    provider
}
