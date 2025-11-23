use exporter::prom::PrometheusRemoteWrite;
use sqlx::PgPool;
use std::sync::{Arc, OnceLock};

/// Global metric exporters instance
static METRIC_EXPORTERS: OnceLock<MetricExporters> = OnceLock::new();

/// Container for metric exporters
/// 
/// This allows scrapers to send metrics to multiple backends.
/// 
/// # Design Pattern
/// 
/// ## Prometheus Remote Write
/// Stored as `Arc<PrometheusRemoteWrite>` because the configuration is immutable
/// and can be shared across all metric exports.
/// 
/// ## TimescaleDB
/// Stored as `Arc<PgPool>` (connection pool) rather than `TimescaleExporter` because:
/// - Each metric export needs **different labels** (target, zone, job, etc.)
/// - Labels are specific to each probe execution and cannot be shared
/// - Creating exporter instances on-demand avoids storing unused instances
/// - The pool is thread-safe and designed to be shared
/// 
/// During initialization, a temporary `TimescaleExporter` is created solely
/// to initialize the database schema (create tables and hypertables), then
/// discarded. Each metric export creates its own exporter with appropriate labels.
/// 
/// # Usage Pattern
/// 
/// ```rust,ignore
/// if let Some(pool) = &exporters.timescale_pool {
///     let exporter = TimescaleExporter::new(pool.clone(), labels);
///     exporter.export(ProbeType::Http, request)?;
/// }
/// ```
#[derive(Clone)]
pub struct MetricExporters {
    pub prometheus_remote_write: Option<Arc<PrometheusRemoteWrite>>,
    pub timescale_pool: Option<Arc<PgPool>>,
}

impl MetricExporters {
    pub fn new(
        prometheus_remote_write: Option<PrometheusRemoteWrite>,
        timescale_pool: Option<Arc<PgPool>>,
    ) -> Self {
        Self {
            prometheus_remote_write: prometheus_remote_write.map(Arc::new),
            timescale_pool,
        }
    }

    pub fn has_exporters(&self) -> bool {
        self.prometheus_remote_write.is_some() || self.timescale_pool.is_some()
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
