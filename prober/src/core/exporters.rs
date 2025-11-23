use exporter::prom::PrometheusRemoteWrite;
use exporter::timescale::TimescaleExporter;
use sqlx::PgPool;
use std::sync::{Arc, OnceLock};

/// Global metric exporters instance
static METRIC_EXPORTERS: OnceLock<MetricExporters> = OnceLock::new();

/// Container for metric exporters
/// This allows scrapers to send metrics to multiple backends
#[derive(Clone)]
pub struct MetricExporters {
    pub prometheus_remote_write: Option<Arc<PrometheusRemoteWrite>>,
    pub timescale: Option<Arc<TimescaleExporter>>,
}

impl MetricExporters {
    pub fn new(
        prometheus_remote_write: Option<PrometheusRemoteWrite>,
        timescale: Option<TimescaleExporter>,
    ) -> Self {
        Self {
            prometheus_remote_write: prometheus_remote_write.map(Arc::new),
            timescale: timescale.map(Arc::new),
        }
    }

    pub fn has_exporters(&self) -> bool {
        self.prometheus_remote_write.is_some() || self.timescale.is_some()
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

/// Create TimescaleDB connection pool
pub async fn create_timescale_pool(connection_string: &str) -> Result<PgPool, sqlx::Error> {
    PgPool::connect(connection_string).await
}
