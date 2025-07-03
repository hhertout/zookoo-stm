use futures::future::join_all;

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
