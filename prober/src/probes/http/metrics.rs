use std::collections::HashMap;
use std::sync::Arc;

use exporter::Export;
use crate::{
    core::MetricExportable,
    probes::http::{
        dns::DnsMetrics,
        request::HttpMetrics,
        tls::TlsMetrics,
    },
};

pub struct HttpRequestMetrics {
    pub dns: DnsMetrics,
    pub http: HttpMetrics,
    pub tls: Option<TlsMetrics>,
    pub labels: Option<Arc<HashMap<String, String>>>,
}

impl HttpRequestMetrics {
    fn extract_metrics_values(&self) -> (u8, u8, u128, u16, u128, Option<u128>, Option<u128>, Option<i64>, Option<i64>) {
        (
            self.http.up,
            self.http.success,
            self.dns.duration.as_millis(),
            self.http.status_code,
            self.http.duration.as_millis(),
            self.tls.as_ref().map(|t| t.duration.as_millis()),
            self.tls.as_ref().map(|t| t.handshake_duration.as_millis()),
            self.tls.as_ref().and_then(|t| t.cert_expiration_date),
            self.tls.as_ref().and_then(|t| t.cert_begin_date),
        )
    }
}

impl MetricExportable for HttpRequestMetrics {
    fn export(&self, target: &str) {
        let mut labels: HashMap<String, String> = HashMap::new();

        labels.insert(String::from("target"), target.to_string());
        labels.insert(String::from("status_code"), self.http.status_code.to_string());

        if let Some(http_version) = self.http.http_version {
            labels.insert(String::from("http_version"), http_version.to_string());
        }

        match self.tls.as_ref() {
            Some(tls_metrics) => {
                // https request with tls metrics
                labels.extend(tls_metrics.to_labels());
                labels.insert(String::from("tls_version"), tls_metrics.version.to_string());

                if let Some(l) = &self.labels {
                    labels.extend(l.as_ref().iter().map(|(k, v)| (k.clone(), v.clone())));
                }

                let exporter = exporter::otel::metrics::MetricsExporter::new(labels.clone());

                // Build metrics HashMap for Export trait
                let mut metrics_map = HashMap::new();
                metrics_map.insert("up".to_string(), self.http.up as isize);
                metrics_map.insert("success".to_string(), self.http.success as isize);
                metrics_map.insert("dns_duration_ms".to_string(), self.dns.duration.as_millis() as isize);
                metrics_map.insert("status_code".to_string(), self.http.status_code as isize);
                metrics_map.insert("http_duration_ms".to_string(), self.http.duration.as_millis() as isize);
                metrics_map.insert("tls_duration_ms".to_string(), tls_metrics.duration.as_millis() as isize);
                metrics_map.insert("tls_handshake_ms".to_string(), tls_metrics.handshake_duration.as_millis() as isize);
                if let Some(cert_exp) = tls_metrics.cert_expiration_date {
                    metrics_map.insert("cert_expiration_ts".to_string(), cert_exp as isize);
                }
                if let Some(cert_begin) = tls_metrics.cert_begin_date {
                    metrics_map.insert("cert_begin_ts".to_string(), cert_begin as isize);
                }

                let request = exporter::ExporterRequest {
                    exporter: exporter::ExporterConfigurationRequest {},
                    metrics: metrics_map,
                };

                if let Err(e) = exporter.export(exporter::ProbeType::Http, request) {
                    log::error!("Failed to export HTTPS metrics to OTEL: {}", e);
                }
            }
            None => {
                // http request without tls metrics
                if let Some(l) = &self.labels {
                    labels.extend(l.as_ref().iter().map(|(k, v)| (k.clone(), v.clone())));
                }

                let exporter = exporter::otel::metrics::MetricsExporter::new(labels.clone());

                // Build metrics HashMap for Export trait
                let mut metrics_map = HashMap::new();
                metrics_map.insert("up".to_string(), self.http.up as isize);
                metrics_map.insert("success".to_string(), self.http.success as isize);
                metrics_map.insert("dns_duration_ms".to_string(), self.dns.duration.as_millis() as isize);
                metrics_map.insert("status_code".to_string(), self.http.status_code as isize);
                metrics_map.insert("http_duration_ms".to_string(), self.http.duration.as_millis() as isize);

                let request = exporter::ExporterRequest {
                    exporter: exporter::ExporterConfigurationRequest {},
                    metrics: metrics_map,
                };

                if let Err(e) = exporter.export(exporter::ProbeType::Http, request) {
                    log::error!("Failed to export HTTP metrics to OTEL: {}", e);
                }
            }
        }

        // Export to configured exporters using the Export trait
        if let Some(exporters) = crate::core::MetricExporters::global() {
            let (up, success, dns_duration, status_code, http_duration, tls_duration, tls_handshake, cert_exp, cert_begin) = 
                self.extract_metrics_values();

            // Build metrics HashMap for Export trait
            let mut metrics_map = HashMap::new();
            metrics_map.insert("up".to_string(), up as isize);
            metrics_map.insert("success".to_string(), success as isize);
            metrics_map.insert("dns_duration_ms".to_string(), dns_duration as isize);
            metrics_map.insert("status_code".to_string(), status_code as isize);
            metrics_map.insert("http_duration_ms".to_string(), http_duration as isize);
            if let Some(tls_dur) = tls_duration {
                metrics_map.insert("tls_duration_ms".to_string(), tls_dur as isize);
            }
            if let Some(tls_hand) = tls_handshake {
                metrics_map.insert("tls_handshake_ms".to_string(), tls_hand as isize);
            }
            if let Some(cert_exp_ts) = cert_exp {
                metrics_map.insert("cert_expiration_ts".to_string(), cert_exp_ts as isize);
            }
            if let Some(cert_begin_ts) = cert_begin {
                metrics_map.insert("cert_begin_ts".to_string(), cert_begin_ts as isize);
            }

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
                if let Err(e) = prom_exporter.export(exporter::ProbeType::Http, req) {
                    log::error!("Failed to export HTTP metrics to Prometheus: {}", e);
                }
            }

            // Export to TimescaleDB if configured
            if let Some(timescale) = &exporters.timescale {
                let ts_exporter = exporter::timescale::TimescaleExporter::new(
                    timescale.pool.clone(),
                    labels.clone(),
                );
                if let Err(e) = ts_exporter.export(exporter::ProbeType::Http, request) {
                    log::error!("Failed to export HTTP metrics to TimescaleDB: {}", e);
                }
            }
        }
    }
}
