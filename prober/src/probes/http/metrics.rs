use std::collections::HashMap;
use std::sync::Arc;

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

                exporter.export_metrics(
                    self.http.up,
                    self.http.success,
                    self.dns.duration.as_millis(),
                    self.http.status_code,
                    self.http.duration.as_millis(),
                    Some(tls_metrics.duration.as_millis()),
                    Some(tls_metrics.handshake_duration.as_millis()),
                    tls_metrics.cert_expiration_date,
                    tls_metrics.cert_begin_date,
                );
            }
            None => {
                // http request without tls metrics
                if let Some(l) = &self.labels {
                    labels.extend(l.as_ref().iter().map(|(k, v)| (k.clone(), v.clone())));
                }

                let exporter = exporter::otel::metrics::MetricsExporter::new(labels.clone());
                exporter.export_metrics(
                    self.http.up,
                    self.http.success,
                    self.dns.duration.as_millis(),
                    self.http.status_code,
                    self.http.duration.as_millis(),
                    None,
                    None,
                    None,
                    None,
                );
            }
        }

        // Export to Prometheus remote_write if configured
        if let Some(exporters) = crate::core::MetricExporters::global() {
            if let Some(remote_write) = &exporters.prometheus_remote_write {
                let prom_exporter = exporter::prom::PrometheusRemoteWriteExporter::new(
                    labels.clone(),
                    Arc::clone(remote_write),
                );
                
                let (up, success, dns_duration, status_code, http_duration, tls_duration, tls_handshake, cert_exp, cert_begin) = 
                    self.extract_metrics_values();

                tokio::spawn(async move {
                    prom_exporter.export_metrics(
                        up,
                        success,
                        dns_duration,
                        status_code,
                        http_duration,
                        tls_duration,
                        tls_handshake,
                        cert_exp,
                        cert_begin,
                    ).await;
                });
            }
        }
    }
}
