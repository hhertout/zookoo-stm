use std::{collections::HashMap, sync::Arc};

use configuration::model::{Configuration, exporter::OtelGrpcExporterConfiguration};
use tokio::sync::RwLock;

use crate::{
    Exporter, ExportersMap, MetricData, ProbeType, labels,
    otlp::{
        meter_provider::init_meter_provider,
        metrics::{HttpMetricsParams, MetricsExporter},
    },
    types::ExporterType,
};

pub struct OtelExporter {
    config: OtelGrpcExporterConfiguration,
    metric_exporter: MetricsExporter,
}

impl OtelExporter {
    pub fn new(
        config: OtelGrpcExporterConfiguration,
        labels: HashMap<String, String>,
        metric_prefix: Option<String>,
    ) -> Self {
        OtelExporter { config, metric_exporter: MetricsExporter::new(labels, metric_prefix, None) }
    }

    /// Initialize observability (tracing and metrics providers)
    fn init_meter_exporter(&mut self) {
        let metric_provider = init_meter_provider(
            self.config.clone(),
            "zookoo".to_string(),
            "dev".to_string(),
            Some("gbl".to_string()),
        );
        self.metric_exporter.set_meter_provider(metric_provider);
    }
}

impl Drop for OtelExporter {
    fn drop(&mut self) {
        if let Some(meter_provider) = &self.metric_exporter.meter_provider {
            let _ = meter_provider.shutdown();
        }
    }
}

impl Exporter for OtelExporter {
    fn get_exporter_type(&self) -> ExporterType {
        ExporterType::Otel
    }

    fn initialize(&mut self) {
        log::info!("inializing OTEL exporter");
        self.init_meter_exporter();
    }

    fn build(config: &Configuration, exporters: &mut ExportersMap) {
        let exporter_wrapper = match config.exporter {
            Some(ref wrapper) => wrapper,
            None => {
                log::info!("no exporters configured");
                return;
            }
        };

        for (label, otel_config) in &exporter_wrapper.otlp {
            let key = format!("exporter.otlp.{}", label);
            log::info!("event=create_exporter type=otlp key={} endpoint={}", key, otel_config.url);

            let mut override_labels: HashMap<String, String> = HashMap::new();
            override_labels.insert("exporter".to_string(), label.clone());
            let mut labels = labels::set_defaults_labels(&config.defaults, override_labels);
            labels::sanitize_labels(&mut labels);

            // Prefix from exporter config takes precedence over default config
            let prefix =
                otel_config.metric_prefix.clone().or_else(|| config.defaults.metric_prefix.clone());

            log::info!(
                "event=otlp_meter_provider_created exporter={} endpoint={}",
                label,
                otel_config.url
            );

            let exporter = OtelExporter::new(otel_config.clone(), labels, prefix);
            exporters.insert(key, Arc::new(RwLock::new(exporter)));
        }
    }

    fn export(&self, probe_type: ProbeType, metric_data: MetricData) {
        let metrics = &metric_data.metrics;
        let target_labels = &metric_data.labels;

        match probe_type {
            ProbeType::Http => {
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
                let up = metrics.get("up").copied().unwrap_or(0) as u8;
                let rtt_ms = metrics.get("rtt_ms").copied().unwrap_or(0) as u128;

                self.metric_exporter.export_icmp_metrics(up, rtt_ms, target_labels);
            }
            ProbeType::Tcp => {
                let up = metrics.get("up").copied().unwrap_or(0) as u8;
                let rtt_ms = metrics.get("rtt_ms").copied().unwrap_or(0) as u128;
                self.metric_exporter.export_icmp_metrics(up, rtt_ms, target_labels);
            }
        }
    }
}
