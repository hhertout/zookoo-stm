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

use std::env;
use std::sync::OnceLock;

use opentelemetry::global::{self, BoxedSpan, BoxedTracer};
use opentelemetry::trace::{Span, SpanKind, TraceContextExt, Tracer};
use opentelemetry::{Context, KeyValue};
use opentelemetry_otlp::{SpanExporter, WithExportConfig};
use opentelemetry_sdk::{Resource, propagation::TraceContextPropagator, trace::SdkTracerProvider};
use tokio::sync::mpsc;

use crate::config::exporter::OtelGrpcExporterConfiguration;
use crate::config::target::{HttpTarget, IcmpTarget};
use crate::scrap_config::ProbeConfig;
use crate::target::http::scrape::HttpScrapper;
use crate::target::icmp::scrape::IcmpScrapper;
use crate::target::scrape_with_shutdown;

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

    // group by interval to spawn one job for each
    let icmp_group_by = config.icmp_group_by_interval();
    let http_group_by = config.http_group_by_interval();

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
                .with_service_name("zookoo")
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
