use std::env;
use std::sync::OnceLock;

use futures::future::join_all;
use opentelemetry::global::{self, BoxedSpan, BoxedTracer};
use opentelemetry::trace::{Span, SpanKind, Tracer};
use opentelemetry::{Context, KeyValue};
use opentelemetry_otlp::{SpanExporter, WithExportConfig};
use opentelemetry_sdk::{Resource, propagation::TraceContextPropagator, trace::SdkTracerProvider};

use crate::config::exporter::OtelGrpcExporterConfiguration;
use crate::scrap_config::ProbeConfig;
use crate::target::http::scrape::http_scrape;

pub(crate) mod config;
pub(crate) mod file;
pub(crate) mod group_by_interval;
pub(crate) mod metrics;
pub mod scrap_config;
pub(crate) mod target;

pub async fn start_probe(config: ProbeConfig) {
    // Enable tracing monitoring
    if env::var("ENABLE_SELF_MONITORING").unwrap_or(String::from("false")) == String::from("true") {
        init_tracer_provider();
    }

    if let Some(otel_conf) = &config.config.exporter.otel {
        let config = OtelGrpcExporterConfiguration::from(otel_conf.clone());
        let _ = exporter::otel::init_metrics_exporter(config.into());
    }

    let handles = http_scrape(config).await;
    let _ = join_all(handles).await;

    /* if let Err(err) = exporter::otel::shutdown(meter_provider) {
        log::error!("exporter shutdown failed = {err}")
    }; */
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

fn tracing_new_span_with_context(
    tracer: &'static BoxedTracer,
    name: String,
    cx: Context,
) -> impl Span {
    tracer.start_with_context(name, &cx)
}

fn init_tracer_provider() {
    let exporter = SpanExporter::builder()
        .with_tonic()
        .with_endpoint(
            env::var("INTERNAL_OLTP_ENDPOINT")
                .unwrap_or("http://localhost:4317/v1/traces".to_string()),
        )
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
    global::set_tracer_provider(provider);
}
