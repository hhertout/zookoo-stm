use std::{collections::HashMap, sync::Arc, time::Duration};

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
        exporter.export_icmp_metrics(self.up, self.duration.as_millis());

        // Export to Prometheus remote_write if configured
        if let Some(exporters) = crate::core::MetricExporters::global() {
            if let Some(remote_write) = &exporters.prometheus_remote_write {
                let prom_exporter = exporter::prom::PrometheusRemoteWriteExporter::new(
                    labels.clone(),
                    Arc::clone(remote_write),
                );

                let (up, duration) = self.extract_metrics_values();

                tokio::spawn(async move {
                    prom_exporter.export_icmp_metrics(up, duration).await;
                });
            }
        }
    }
}
