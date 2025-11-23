use std::{collections::HashMap, sync::Arc, time::Duration};

use exporter::Export;
use crate::core::MetricExportable;

pub struct IcmpRequestMetrics {
    pub up: u8,
    pub duration: Duration,
    pub labels: Option<Arc<HashMap<String, String>>>,
}

impl IcmpRequestMetrics {
    fn extract_metrics_values(&self) -> (u8, u128) {
        (self.up, self.duration.as_millis())
    }
}

impl MetricExportable for IcmpRequestMetrics {
    fn export(&self, target: &str) {
        let mut labels: HashMap<String, String> = HashMap::new();
        labels.insert(String::from("target"), target.to_string());

        if let Some(l) = &self.labels {
            labels.extend(l.as_ref().iter().map(|(k, v)| (k.clone(), v.clone())));
        }

        let exporter = exporter::otel::metrics::MetricsExporter::new(labels.clone());

        // Build metrics HashMap for Export trait
        let mut otel_metrics_map = HashMap::new();
        otel_metrics_map.insert("up".to_string(), self.up as isize);
        otel_metrics_map.insert("rtt_ms".to_string(), self.duration.as_millis() as isize);

        let otel_request = exporter::ExporterRequest {
            exporter: exporter::ExporterConfigurationRequest {},
            metrics: otel_metrics_map,
        };

        if let Err(e) = exporter.export(exporter::ProbeType::Icmp, otel_request) {
            log::error!("Failed to export ICMP metrics to OTEL: {}", e);
        }

        // Export to configured exporters using the Export trait
        if let Some(exporters) = crate::core::MetricExporters::global() {
            let (up, duration) = self.extract_metrics_values();

            // Build metrics HashMap for Export trait
            let mut metrics_map = HashMap::new();
            metrics_map.insert("up".to_string(), up as isize);
            metrics_map.insert("rtt_ms".to_string(), duration as isize);

            let request = exporter::ExporterRequest {
                exporter: exporter::ExporterConfigurationRequest {},
                metrics: metrics_map,
            };

            // Export to Prometheus remote_write if configured
            if let Some(remote_write) = &exporters.prometheus_remote_write {
                let prom_exporter = exporter::prom::PrometheusRemoteWriteExporter::new(
                    labels.clone(),
                    Arc::clone(remote_write),
                );
                let req = request.clone();
                if let Err(e) = prom_exporter.export(exporter::ProbeType::Icmp, req) {
                    log::error!("Failed to export ICMP metrics to Prometheus: {}", e);
                }
            }

            // Export to TimescaleDB if configured
            if let Some(timescale) = &exporters.timescale {
                let ts_exporter = exporter::timescale::TimescaleExporter::new(
                    timescale.pool.clone(),
                    labels.clone(),
                );
                if let Err(e) = ts_exporter.export(exporter::ProbeType::Icmp, request) {
                    log::error!("Failed to export ICMP metrics to TimescaleDB: {}", e);
                }
            }
        }
    }
}
