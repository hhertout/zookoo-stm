use std::collections::HashMap;

use crate::{
    metrics::MetricExportable,
    target::http::{dns::DnsMetrics, request::HttpMetrics, tls::TlsMetrics},
};

pub struct HttpRequestMetrics {
    pub dns: DnsMetrics,
    pub http: HttpMetrics,
    pub tls: Option<TlsMetrics>,
    pub labels: Option<HashMap<String, String>>,
}

impl MetricExportable for HttpRequestMetrics {
    fn export(&self, target: &str) {
        let mut labels: HashMap<String, String> = HashMap::new();

        labels.insert(String::from("target"), target.to_string());

        if let Some(http_version) = self.http.http_version {
            labels.insert(String::from("http_version"), http_version.to_string());
        }

        match self.tls.as_ref() {
            Some(tls_metrics) => {
                // https request with tls metrics
                labels.extend(tls_metrics.to_labels());
                labels.insert(String::from("tls_veriosn"), tls_metrics.version.to_string());

                if let Some(l) = self.labels.clone() {
                    labels.extend(l);
                }

                let exporter = exporter::otel::metrics::MetricsExporter::new(labels);

                exporter.export_metrics(
                    self.http.up,
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
                if let Some(l) = self.labels.clone() {
                    labels.extend(l);
                }

                let exporter = exporter::otel::metrics::MetricsExporter::new(labels);
                exporter.export_metrics(
                    self.http.up,
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
    }
}
