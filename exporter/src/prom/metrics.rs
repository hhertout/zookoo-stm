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
    prefix: String,
    default_labels: HashMap<String, String>,
    remote_write: Arc<PrometheusRemoteWrite>,
}

impl PrometheusRemoteWriteExporter {
    /// Create a new Prometheus remote_write exporter with the given labels and default prefix
    pub fn new(labels: HashMap<String, String>, remote_write: Arc<PrometheusRemoteWrite>) -> Self {
        Self { prefix: "zookoo_".to_string(), default_labels: labels, remote_write }
    }

    /// Create a new Prometheus remote_write exporter with a custom prefix
    pub fn with_prefix(
        prefix: String,
        labels: HashMap<String, String>,
        remote_write: Arc<PrometheusRemoteWrite>,
    ) -> Self {
        // Ensure prefix ends with underscore
        let prefix = if prefix.ends_with('_') { prefix } else { format!("{}_", prefix) };
        Self { prefix, default_labels: labels, remote_write }
    }

    /// Format a metric name with the configured prefix
    fn metric_name(&self, name: &str) -> String {
        format!("{}{}", self.prefix, name)
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
            (self.metric_name("icmp_target_up"), up as f64, labels.clone()),
            (self.metric_name("icmp_rtt_seconds"), rtt_seconds, labels),
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
                // HTTP metrics - extract all available metrics
                let up = metrics.get("up").copied().unwrap_or(0) as f64;
                let success = metrics.get("success").copied().unwrap_or(0) as f64;
                let status_code = metrics.get("status_code").copied().unwrap_or(0) as f64;

                // Duration metrics (convert ms to seconds)
                let dns_duration =
                    metrics.get("dns_duration_ms").copied().unwrap_or(0) as f64 / 1000.0;
                let tcp_connect_duration =
                    metrics.get("tcp_connect_duration_ms").copied().unwrap_or(0) as f64 / 1000.0;
                let time_to_first_byte =
                    metrics.get("time_to_first_byte_ms").copied().unwrap_or(0) as f64 / 1000.0;
                let content_transfer_duration =
                    metrics.get("content_transfer_duration_ms").copied().unwrap_or(0) as f64
                        / 1000.0;
                let http_duration =
                    metrics.get("http_duration_ms").copied().unwrap_or(0) as f64 / 1000.0;

                let mut metric_vec = vec![
                    // Core metrics
                    (self.metric_name("target_up"), up, labels.clone()),
                    (self.metric_name("target_success"), success, labels.clone()),
                    (self.metric_name("http_status_code"), status_code, labels.clone()),
                    // Duration metrics (gauges in seconds)
                    (
                        self.metric_name("http_request_duration_seconds"),
                        http_duration,
                        labels.clone(),
                    ),
                    (self.metric_name("dns_lookup_duration_seconds"), dns_duration, labels.clone()),
                    (
                        self.metric_name("tcp_connect_duration_seconds"),
                        tcp_connect_duration,
                        labels.clone(),
                    ),
                    (
                        self.metric_name("time_to_first_byte_seconds"),
                        time_to_first_byte,
                        labels.clone(),
                    ),
                    (
                        self.metric_name("content_transfer_duration_seconds"),
                        content_transfer_duration,
                        labels.clone(),
                    ),
                ];

                // Optional TLS metrics
                if let Some(tls_handshake) = metrics.get("tls_handshake_ms") {
                    let tls_duration = *tls_handshake as f64 / 1000.0;
                    metric_vec.push((
                        self.metric_name("tls_handshake_duration_seconds"),
                        tls_duration,
                        labels.clone(),
                    ));
                }

                // Certificate metrics (timestamps)
                if let Some(cert_expiration) = metrics.get("cert_expiration_ts") {
                    metric_vec.push((
                        self.metric_name("cert_expiration_timestamp"),
                        *cert_expiration as f64,
                        labels.clone(),
                    ));
                }
                if let Some(cert_begin) = metrics.get("cert_begin_ts") {
                    metric_vec.push((
                        self.metric_name("cert_begin_timestamp"),
                        *cert_begin as f64,
                        labels.clone(),
                    ));
                }

                log::debug!(
                    "event=prometheus_remote_write_export metrics_count={} success={}",
                    metric_vec.len(),
                    success
                );

                let remote_write = Arc::clone(&self.remote_write);
                tokio::spawn(async move {
                    match remote_write.push_metrics(metric_vec, None).await {
                        Ok(_) => log::debug!("event=prometheus_remote_write_success"),
                        Err(e) => log::error!("event=prometheus_remote_write_error error={}", e),
                    }
                });
            }
            ProbeType::Icmp => {
                // ICMP metrics
                let up = metrics.get("up").copied().unwrap_or(0) as f64;
                let rtt_seconds = metrics.get("rtt_ms").copied().unwrap_or(0) as f64 / 1000.0;

                let metric_vec = vec![
                    (self.metric_name("icmp_target_up"), up, labels.clone()),
                    (self.metric_name("icmp_rtt_seconds"), rtt_seconds, labels),
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
