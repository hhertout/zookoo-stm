use crate::metrics::{http_metrics::HttpRequestMetrics, icmp_metrics::IcmpRequestMetrics};

pub(crate) mod http_metrics;
pub(crate) mod icmp_metrics;

pub enum Metrics {
    Http(HttpRequestMetrics),
    Icmp(IcmpRequestMetrics),
}

pub trait MetricExportable {
    fn export(&self, target: &str);
}
