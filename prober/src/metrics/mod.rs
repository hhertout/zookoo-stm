use crate::metrics::http_metrics::HttpRequestMetrics;

pub(crate) mod http_metrics;

pub enum Metrics {
    Http(HttpRequestMetrics),
}

pub trait MetricExportable {
    fn export(&self, target: &str);
}
