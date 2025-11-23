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
    #[allow(unreachable_patterns)]
    fn export(&self, probe_type: crate::ProbeType, data: ExporterRequest) -> Result<(), Error> {
        use crate::ProbeType;

        match probe_type {
            ProbeType::Http => {
                // HTTP metrics
                let success = data.metrics.get("success").copied().unwrap_or(0) as f64;
                let dns_duration = data.metrics.get("dns_duration_ms").copied().unwrap_or(0) as f64 / 1000.0;
                let http_duration = data.metrics.get("http_duration_ms").copied().unwrap_or(0) as f64 / 1000.0;

                let metrics = vec![
                    ("http_probe_success".to_string(), success, self.labels.clone()),
                    ("http_probe_duration_seconds".to_string(), http_duration, self.labels.clone()),
                    ("http_probe_dns_duration_seconds".to_string(), dns_duration, self.labels.clone()),
                ];

                let remote_write = Arc::clone(&self.remote_write);
                tokio::spawn(async move {
                    if let Err(e) = remote_write.push_metrics(metrics, None).await {
                        log::error!("Failed to export HTTP metrics to Prometheus: {}", e);
                    }
                });

                Ok(())
            }
            ProbeType::Icmp => {
                // ICMP metrics
                let up = data.metrics.get("up").copied().unwrap_or(0) as f64;
                let rtt_ms = data.metrics.get("rtt_ms").copied().unwrap_or(0) as f64 / 1000.0;

                let metrics = vec![
                    ("icmp_probe_success".to_string(), up, self.labels.clone()),
                    ("icmp_probe_rtt_seconds".to_string(), rtt_ms, self.labels.clone()),
                ];

                let remote_write = Arc::clone(&self.remote_write);
                tokio::spawn(async move {
                    if let Err(e) = remote_write.push_metrics(metrics, None).await {
                        log::error!("Failed to export ICMP metrics to Prometheus: {}", e);
                    }
                });

                Ok(())
            }
            _ => {
                Err(Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Prometheus exporter does not support probe type: {}", probe_type)
                ))
            }
        }
    }
}
