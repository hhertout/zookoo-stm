use std::collections::HashMap;
use std::sync::Arc;
use std::io::Error;

use super::remote_write::PrometheusRemoteWrite;
use crate::{Export, ExporterRequest};

/// Prometheus metrics exporter that uses the remote_write API.
/// This is similar to the OTLP MetricsExporter but for Prometheus.
pub struct PrometheusRemoteWriteExporter {
    labels: HashMap<String, String>,
    remote_write: Arc<PrometheusRemoteWrite>,
}

impl PrometheusRemoteWriteExporter {
    /// Create a new Prometheus remote_write exporter with the given labels
    pub fn new(labels: HashMap<String, String>, remote_write: Arc<PrometheusRemoteWrite>) -> Self {
        Self {
            labels,
            remote_write,
        }
    }

    /// Export ICMP metrics to Prometheus remote_write
    pub async fn export_icmp_metrics(&self, up: u8, duration: u128) {
        let rtt_seconds = duration as f64 / 1000.0; // Convert ms to seconds

        let metrics = vec![
            ("icmp_probe_success".to_string(), up as f64, self.labels.clone()),
            ("icmp_probe_rtt_seconds".to_string(), rtt_seconds, self.labels.clone()),
        ];

        if let Err(e) = self.remote_write.push_metrics(metrics, None).await {
            log::error!("Failed to export ICMP metrics to Prometheus remote_write: {}", e);
        }
    }

    /// Export HTTP metrics to Prometheus remote_write
    pub async fn export_metrics(
        &self,
        _up: u8,
        success: u8,
        dns_lookup_duration: u128,
        _http_status_code: u16,
        http_request_duration: u128,
        _http_tls_lookup_duration: Option<u128>,
        _http_tls_handshake_duration: Option<u128>,
        _tls_cert_expiration_ts: Option<i64>,
        _tls_cert_begin_ts: Option<i64>,
    ) {
        let duration_seconds = http_request_duration as f64 / 1000.0; // Convert ms to seconds
        let dns_duration_seconds = dns_lookup_duration as f64 / 1000.0; // Convert ms to seconds

        let metrics = vec![
            ("http_probe_success".to_string(), success as f64, self.labels.clone()),
            ("http_probe_duration_seconds".to_string(), duration_seconds, self.labels.clone()),
            ("http_probe_dns_duration_seconds".to_string(), dns_duration_seconds, self.labels.clone()),
        ];

        if let Err(e) = self.remote_write.push_metrics(metrics, None).await {
            log::error!("Failed to export HTTP metrics to Prometheus remote_write: {}", e);
        }
    }
}

impl Export for PrometheusRemoteWriteExporter {
    fn export(&self, data: ExporterRequest) -> Result<(), Error> {
        // Convert the generic ExporterRequest into Prometheus remote_write format
        let mut metrics = Vec::new();
        
        for (name, value) in data.metrics {
            metrics.push((name, value as f64, self.labels.clone()));
        }

        // We need to spawn a tokio task because Export trait is sync but push_metrics is async
        let remote_write = Arc::clone(&self.remote_write);
        tokio::spawn(async move {
            if let Err(e) = remote_write.push_metrics(metrics, None).await {
                log::error!("Failed to export metrics via Export trait: {}", e);
            }
        });

        Ok(())
    }
}
