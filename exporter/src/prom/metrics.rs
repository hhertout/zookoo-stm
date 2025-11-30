use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct HttpMetricsParams {
    pub up: u8,
    pub success: u8,
    pub dns_lookup_duration: u128,
    pub http_status_code: u16,
    pub http_request_duration: u128,
    pub http_tls_lookup_duration: Option<u128>,
    pub http_tls_handshake_duration: Option<u128>,
    pub tls_cert_expiration_ts: Option<i64>,
    pub tls_cert_begin_ts: Option<i64>,
    pub target_labels: HashMap<String, String>,
}

use super::remote_write::PrometheusRemoteWrite;
use crate::{Exporter, MetricData};

/// Prometheus metrics exporter that uses the remote_write API.
/// This is similar to the OTLP MetricsExporter but for Prometheus.
pub struct PrometheusRemoteWriteExporter {
    default_labels: HashMap<String, String>,
    remote_write: Arc<PrometheusRemoteWrite>,
}

impl PrometheusRemoteWriteExporter {
    /// Create a new Prometheus remote_write exporter with the given labels
    pub fn new(labels: HashMap<String, String>, remote_write: Arc<PrometheusRemoteWrite>) -> Self {
        Self { default_labels: labels, remote_write }
    }

    /// Merge default labels with target-specific labels (target labels take precedence)
    fn merge_labels(&self, target_labels: &HashMap<String, String>) -> HashMap<String, String> {
        let mut merged = self.default_labels.clone();
        for (key, value) in target_labels {
            merged.insert(key.clone(), value.clone());
        }
        merged
    }

    /// Export ICMP metrics to Prometheus remote_write
    pub async fn export_icmp_metrics(
        &self,
        up: u8,
        duration: u128,
        target_labels: &HashMap<String, String>,
    ) {
        let labels = self.merge_labels(target_labels);
        let rtt_seconds = duration as f64 / 1000.0; // Convert ms to seconds

        let metrics = vec![
            ("icmp_probe_success".to_string(), up as f64, labels.clone()),
            ("icmp_probe_rtt_seconds".to_string(), rtt_seconds, labels),
        ];

        if let Err(e) = self.remote_write.push_metrics(metrics, None).await {
            log::error!("Failed to export ICMP metrics to Prometheus remote_write: {}", e);
        }
    }
}

impl Exporter for PrometheusRemoteWriteExporter {
    fn export(&self, probe_type: crate::ProbeType, metric_data: MetricData) {
        use crate::ProbeType;

        let metrics = &metric_data.metrics;
        let labels = self.merge_labels(&metric_data.labels);

        match probe_type {
            ProbeType::Http => {
                // HTTP metrics
                let success = metrics.get("success").copied().unwrap_or(0) as f64;
                let dns_duration =
                    metrics.get("dns_duration_ms").copied().unwrap_or(0) as f64 / 1000.0;
                let http_duration =
                    metrics.get("http_duration_ms").copied().unwrap_or(0) as f64 / 1000.0;

                let metric_vec = vec![
                    ("http_probe_success".to_string(), success, labels.clone()),
                    ("http_probe_duration_seconds".to_string(), http_duration, labels.clone()),
                    ("http_probe_dns_duration_seconds".to_string(), dns_duration, labels),
                ];

                let remote_write = Arc::clone(&self.remote_write);
                tokio::spawn(async move {
                    if let Err(e) = remote_write.push_metrics(metric_vec, None).await {
                        log::error!("Failed to export HTTP metrics to Prometheus: {}", e);
                    }
                });
            }
            ProbeType::Icmp => {
                // ICMP metrics
                let up = metrics.get("up").copied().unwrap_or(0) as f64;
                let rtt_ms = metrics.get("rtt_ms").copied().unwrap_or(0) as f64 / 1000.0;

                let metric_vec = vec![
                    ("icmp_probe_success".to_string(), up, labels.clone()),
                    ("icmp_probe_rtt_seconds".to_string(), rtt_ms, labels),
                ];

                let remote_write = Arc::clone(&self.remote_write);
                tokio::spawn(async move {
                    if let Err(e) = remote_write.push_metrics(metric_vec, None).await {
                        log::error!("Failed to export ICMP metrics to Prometheus: {}", e);
                    }
                });
            }
        }
    }
}
