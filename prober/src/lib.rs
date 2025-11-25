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
//! To start a scraping session, call the run function with a ProbeConfig:
//!
//! ```ignore
//! prober::run(ProbeConfig::from(config)).await;
//! ```
//!
//! The configuration should be compliant with the `configuration` crate already present in the repository.

use opentelemetry_sdk::trace::SdkTracerProvider;
use tokio::sync::mpsc;

use crate::config::exporter::OtelGrpcExporterConfiguration;
use crate::core::scraper::scrape_with_shutdown;
use crate::probes::{HttpScraper, HttpTarget, IcmpScraper, IcmpTarget};
use crate::scrap_config::ProbeConfig;

pub(crate) mod config;
pub(crate) mod core;
pub(crate) mod exporters;
pub(crate) mod observability;
pub(crate) mod probes;
pub mod scrap_config;
pub(crate) mod utils;

pub async fn run(config: ProbeConfig) {
    // Initialize observability (tracing) if self-monitoring is enabled
    let mut tracer_provider: Option<SdkTracerProvider> = None;

    if let Some(scrap_config) = &config.scrap_config.default.self_monitoring {
        if scrap_config.enable {
            let zone = config.scrap_config.default.probe_zone.clone();
            let self_monitoring_conf = scrap_config.clone();
            tracer_provider = Some(observability::init_tracer_provider(
                self_monitoring_conf.otel_endpoint.clone(),
                self_monitoring_conf.service_name.clone(),
                self_monitoring_conf.env.clone(),
                zone,
            ));
        }
    }

    // Initialize OpenTelemetry metrics exporter if configured
    if let Some(otel_conf) = &config.scrap_config.exporter.otel {
        let config = OtelGrpcExporterConfiguration::from(otel_conf.clone());
        let _ = exporter::otel::init_metrics_exporter(config.into());
        log::warn!("otel exporter is enabled");
    }

    // Initialize Prometheus remote_write exporter if configured (for Grafana Alloy, Prometheus, Mimir, etc.)
    let prometheus_remote_write =
        if let Some(rw_conf) = &config.scrap_config.exporter.prometheus_remote_write {
            log::warn!("prometheus remote_write exporter is enabled");
            log::info!("prometheus remote_write url: {}", rw_conf.url);
            log::info!("prometheus job: {}", rw_conf.job);

            // Note: extra_labels are for global labels like environment, region, etc.
            // Zone is already included in each target's labels via apply_default_labels()
            let extra_labels = std::collections::HashMap::new();

            let rw_config = exporter::prom::PrometheusRemoteWriteConfig {
                url: rw_conf.url.clone(),
                job: rw_conf.job.clone(),
                instance: rw_conf.instance.clone(),
                auth: rw_conf
                    .auth
                    .as_ref()
                    .map(|a| exporter::config::AuthConfiguration {
                        username: a.username.clone(),
                        password: a.password.clone(),
                        bearer: a.bearer.clone(),
                    }),
                extra_labels,
            };

            match exporter::prom::PrometheusRemoteWrite::new(rw_config) {
                Ok(exporter) => Some(exporter),
                Err(e) => {
                    log::error!(
                        "Failed to initialize Prometheus remote_write exporter: {}",
                        e
                    );
                    None
                }
            }
        } else {
            None
        };

    // Initialize TimescaleDB pool if configured
    let (timescale_pool, timescale_schema) =
        if let Some(ts_conf) = &config.scrap_config.exporter.timescale {
            log::warn!("timescale exporter is enabled");
            log::info!("timescale connection: {}", ts_conf.connection_string);
            log::info!("timescale schema: {}", ts_conf.schema);

            match crate::core::exporters::create_timescale_pool(&ts_conf.connection_string).await {
                Ok(pool) => {
                    let pool = std::sync::Arc::new(pool);

                    // Initialize schema (creates tables and hypertables)
                    // Use a temporary exporter with empty labels just for schema initialization
                    let schema_initializer = exporter::timescale::TimescaleExporter::with_schema(
                        pool.clone(),
                        std::collections::HashMap::new(),
                        ts_conf.schema.clone(),
                    );

                    match schema_initializer.init_schema().await {
                        Ok(_) => {
                            log::info!("TimescaleDB schema initialized successfully");
                            (Some(pool), ts_conf.schema.clone())
                        }
                        Err(e) => {
                            log::error!("Failed to initialize TimescaleDB schema: {}", e);
                            (None, "public".to_string())
                        }
                    }
                }
                Err(e) => {
                    log::error!("Failed to connect to TimescaleDB: {}", e);
                    (None, "public".to_string())
                }
            }
        } else {
            (None, "public".to_string())
        };

    // Create exporters container and initialize as global
    let exporters = crate::core::MetricExporters::new(
        prometheus_remote_write,
        timescale_pool,
        timescale_schema,
    );
    exporters.clone().init_global();

    //
    // Run the probe and start the scraping
    //
    launch_probe_engine(config, exporters).await;

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
pub async fn launch_probe_engine(mut config: ProbeConfig, exporters: crate::core::MetricExporters) {
    let (icmp_shutdown_tx, icmp_shutdown_rx) = mpsc::channel::<()>(1);
    let (http_shutdown_tx, http_shutdown_rx) = mpsc::channel::<()>(1);

    // group by interval to spawn one job for each
    let icmp_group_by = config.apply_default_labels().icmp_group_by_interval();
    let http_group_by = config.apply_default_labels().http_group_by_interval();

    log::warn!(
        "ICMP targets: s5={} s10={} s30={} m1={}",
        icmp_group_by.s5.len(),
        icmp_group_by.s10.len(),
        icmp_group_by.s30.len(),
        icmp_group_by.m1.len()
    );
    log::warn!(
        "HTTP targets: s5={} s10={} s30={} m1={}",
        http_group_by.s5.len(),
        http_group_by.s10.len(),
        http_group_by.s30.len(),
        http_group_by.m1.len()
    );

    // launch the scraping process for each target type
    // for each interval -
    // |_ launch 1 job per interval
    //    |_ each interval launch one job per target
    //    |_ each target job complete send metrics in the same job
    //
    let scrape_http_task = tokio::spawn(scrape_with_shutdown::<HttpScraper, HttpTarget>(
        http_group_by,
        http_shutdown_rx,
        exporters.clone(),
    ));

    let scrape_icmp_task = tokio::spawn(scrape_with_shutdown::<IcmpScraper, IcmpTarget>(
        icmp_group_by,
        icmp_shutdown_rx,
        exporters,
    ));

    //
    // waiting for the process stop
    //
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for Ctrl+C");

    log::info!("Ctrl+C received, shutting down...");

    // clean close of the jobs
    let _ = icmp_shutdown_tx.send(()).await;
    let _ = http_shutdown_tx.send(()).await;
    let _ = scrape_http_task.await;
    let _ = scrape_icmp_task.await;
}
