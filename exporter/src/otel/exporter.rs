use std::{collections::HashMap, sync::Arc};

use configuration::model::Configuration;

use crate::{
    Exporter, ExportersMap, MetricData, ProbeType, labels,
    otel::metrics::{HttpMetricsParams, MetricsExporter},
};

pub struct OtelExporter {
    metric_exporter: MetricsExporter,
}

impl OtelExporter {
    pub fn new(labels: HashMap<String, String>) -> Self {
        OtelExporter { metric_exporter: MetricsExporter::new(labels) }
    }
}

impl Exporter for OtelExporter {
    fn build(config: &Configuration, exporters: &mut ExportersMap) {
        let exporter_wrapper = match config.exporter {
            Some(ref wrapper) => wrapper,
            None => {
                log::info!("no exporters configured");
                return;
            }
        };
        for (label, otel_config) in &exporter_wrapper.otel {
            let key = format!("exporter.otel.{}", label);
            log::info!("event=create_exporter type=otel key={} endpoint={}", key, otel_config.url);

            let mut override_labels: HashMap<String, String> = HashMap::new();
            override_labels.insert("exporter".to_string(), label.clone());
            let mut labels = labels::set_defaults_labels(&config.defaults, override_labels);
            labels::sanitize_labels(&mut labels);

            let exporter = OtelExporter::new(labels);
            exporters.insert(key, Arc::new(exporter));
        }
    }

    fn export(&self, probe_type: ProbeType, metric_data: MetricData) {
        let metrics = &metric_data.metrics;
        let target_labels = &metric_data.labels;

        match probe_type {
            ProbeType::Http => {
                // Extract HTTP metrics from the HashMap
                let up = metrics.get("up").copied().unwrap_or(0) as u8;
                let success = metrics.get("success").copied().unwrap_or(0) as u8;
                let dns_duration = metrics.get("dns_duration_ms").copied().unwrap_or(0) as u128;
                let tcp_connect_duration =
                    metrics.get("tcp_connect_duration_ms").copied().unwrap_or(0) as u128;
                let time_to_first_byte =
                    metrics.get("time_to_first_byte_ms").copied().unwrap_or(0) as u128;
                let content_transfer_duration =
                    metrics.get("content_transfer_duration_ms").copied().unwrap_or(0) as u128;
                let status_code = metrics.get("status_code").copied().unwrap_or(0) as u16;
                let http_duration = metrics.get("http_duration_ms").copied().unwrap_or(0) as u128;
                let tls_duration = metrics.get("tls_duration_ms").map(|v| *v as u128);
                let tls_handshake = metrics.get("tls_handshake_ms").map(|v| *v as u128);
                let cert_expiration = metrics.get("cert_expiration_ts").map(|v| *v as i64);
                let cert_begin = metrics.get("cert_begin_ts").map(|v| *v as i64);

                let to_export = HttpMetricsParams {
                    up,
                    success,
                    dns_lookup_duration: dns_duration,
                    tcp_connect_duration,
                    time_to_first_byte,
                    content_transfer_duration,
                    http_status_code: status_code,
                    http_request_duration: http_duration,
                    http_tls_lookup_duration: tls_duration,
                    http_tls_handshake_duration: tls_handshake,
                    tls_cert_expiration_ts: cert_expiration,
                    tls_cert_begin_ts: cert_begin,
                    target_labels: target_labels.clone(),
                };
                self.metric_exporter.export_http_metrics(to_export);
            }
            ProbeType::Icmp => {
                // Extract ICMP metrics from the HashMap
                let up = metrics.get("up").copied().unwrap_or(0) as u8;
                let rtt_ms = metrics.get("rtt_ms").copied().unwrap_or(0) as u128;

                self.metric_exporter.export_icmp_metrics(up, rtt_ms, target_labels);
            }
        }
    }
}
