use exporter::prom::PrometheusRemoteWrite;
use std::sync::{Arc, OnceLock};

/// Global metric exporters instance
static METRIC_EXPORTERS: OnceLock<MetricExporters> = OnceLock::new();

/// Container for metric exporters
/// This allows scrapers to send metrics to multiple backends
#[derive(Clone)]
pub struct MetricExporters {
    pub prometheus_remote_write: Option<Arc<PrometheusRemoteWrite>>,
}

impl MetricExporters {
    pub fn new(
        prometheus_remote_write: Option<PrometheusRemoteWrite>,
    ) -> Self {
        Self {
            prometheus_remote_write: prometheus_remote_write.map(Arc::new),
        }
    }

    pub fn has_exporters(&self) -> bool {
        self.prometheus_remote_write.is_some()
    }

    /// Initialize the global exporters instance
    pub fn init_global(self) {
        let _ = METRIC_EXPORTERS.set(self);
    }

    /// Get the global exporters instance
    pub fn global() -> Option<&'static MetricExporters> {
        METRIC_EXPORTERS.get()
    }
}
