use crate::timescale::repository::{TimescaleRepository, HttpMetricRow, IcmpMetricRow};
use crate::{Export, ExporterRequest};
use sqlx::PgPool;
use std::collections::HashMap;
use std::io::Error;
use std::sync::Arc;

/// Safe conversion from u128 to i64 for duration metrics
/// 
/// # Design Rationale
/// 
/// Duration metrics are internally represented as `u128` (unsigned 128-bit) in the prober
/// for maximum precision and to avoid overflow during intermediate calculations. However,
/// TimescaleDB/PostgreSQL uses `BIGINT` (signed 64-bit) for numeric storage.
/// 
/// # Storage Capacity
/// 
/// An `i64` can represent values from `-2^63` to `2^63-1`:
/// - Maximum: 9,223,372,036,854,775,807 milliseconds
/// - Equivalent: ~292 million years
/// - Practical use: Network timeouts are typically 1-120 seconds
/// 
/// # Overflow Protection
/// 
/// While overflow is theoretically possible, it is **extremely unlikely** in practice:
/// - A network probe timing out at 292 million years would indicate a system failure
/// - Modern HTTP clients timeout after 30-120 seconds by default
/// - Even pathological cases won't approach this limit
/// 
/// This function uses `try_into()` with fallback to `i64::MAX` to guarantee:
/// 1. **No panics**: Checked conversion prevents runtime crashes
/// 2. **Data preservation**: Normal values (99.99999...%) convert without loss
/// 3. **Graceful degradation**: Overflow cases are logged and clamped
/// 4. **Observable behavior**: Log warnings help detect edge cases in production
/// 
/// # Examples
/// 
/// ```rust,ignore
/// // Normal probe duration: 1 second
/// assert_eq!(duration_to_i64(1000), 1000);
/// 
/// // Long timeout: 5 minutes
/// assert_eq!(duration_to_i64(300_000), 300_000);
/// 
/// // Overflow case (hypothetical): clamps to max
/// let overflow = (i64::MAX as u128) + 1;
/// assert_eq!(duration_to_i64(overflow), i64::MAX); // Logs warning
/// ```
/// 
/// # Testing
/// 
/// See `timescale::metrics_tests::test_duration_to_i64_*` for comprehensive test coverage.
#[inline]
pub(crate) fn duration_to_i64(duration: u128) -> i64 {
    duration.try_into().unwrap_or_else(|_| {
        log::warn!("Duration overflow detected: {} ms exceeds i64::MAX, clamping to maximum value", duration);
        i64::MAX
    })
}

/// TimescaleDB metrics exporter
/// 
/// Stores probe metrics in TimescaleDB hypertables for time-series analysis.
pub struct TimescaleExporter {
    pub pool: Arc<PgPool>,
    repository: TimescaleRepository,
    labels: HashMap<String, String>,
}

impl TimescaleExporter {
    /// Create a new TimescaleDB exporter with the given connection pool and labels
    /// Uses the default "public" schema
    pub fn new(pool: Arc<PgPool>, labels: HashMap<String, String>) -> Self {
        let repository = TimescaleRepository::new(pool.clone());
        Self { pool, repository, labels }
    }

    /// Create a new TimescaleDB exporter with a custom schema
    pub fn with_schema(pool: Arc<PgPool>, labels: HashMap<String, String>, schema: String) -> Self {
        let repository = TimescaleRepository::with_schema(pool.clone(), schema);
        Self { pool, repository, labels }
    }

    /// Initialize database schema and hypertables
    pub async fn init_schema(&self) -> Result<(), sqlx::Error> {
        // Create HTTP metrics table and hypertable
        self.repository.create_http_metrics_table().await?;
        self.repository.create_http_hypertable().await?;
        self.repository.create_http_metrics_index().await?;

        // Create ICMP metrics table and hypertable
        self.repository.create_icmp_metrics_table().await?;
        self.repository.create_icmp_hypertable().await?;
        self.repository.create_icmp_metrics_index().await?;

        log::info!("TimescaleDB schema initialized successfully");
        Ok(())
    }

    /// Export HTTP metrics to TimescaleDB
    pub async fn export_http_metrics(
        &self,
        up: u8,
        success: u8,
        dns_duration: u128,
        status_code: u16,
        http_duration: u128,
        tls_duration: Option<u128>,
        tls_handshake: Option<u128>,
        cert_expiration_ts: Option<i64>,
        cert_begin_ts: Option<i64>,
    ) -> Result<(), sqlx::Error> {
        let target = self.labels.get("target").map(|s| s.as_str()).unwrap_or("unknown");
        let zone = self.labels.get("zone").map(|s| s.as_str());
        let job = self.labels.get("job").map(|s| s.as_str());
        let http_version = self.labels.get("http_version").map(|s| s.as_str());
        let tls_version = self.labels.get("tls_version").map(|s| s.as_str());

        let labels_json = serde_json::to_value(&self.labels)
            .unwrap_or(serde_json::Value::Null);

        self.repository.insert_http_metrics(
            target,
            zone,
            job,
            up as i16,
            success as i16,
            status_code as i32,
            duration_to_i64(dns_duration),
            duration_to_i64(http_duration),
            tls_duration.map(duration_to_i64),
            tls_handshake.map(duration_to_i64),
            cert_expiration_ts,
            cert_begin_ts,
            http_version,
            tls_version,
            labels_json,
        ).await
    }

    /// Export ICMP metrics to TimescaleDB
    pub async fn export_icmp_metrics(
        &self,
        up: u8,
        rtt_ms: u128,
    ) -> Result<(), sqlx::Error> {
        let target = self.labels.get("target").map(|s| s.as_str()).unwrap_or("unknown");
        let zone = self.labels.get("zone").map(|s| s.as_str());
        let job = self.labels.get("job").map(|s| s.as_str());

        let labels_json = serde_json::to_value(&self.labels)
            .unwrap_or(serde_json::Value::Null);

        self.repository.insert_icmp_metrics(
            target,
            zone,
            job,
            up as i16,
            duration_to_i64(rtt_ms),
            labels_json,
        ).await
    }

    /// Get the latest HTTP metrics for a target
    pub async fn get_latest_http_metrics(
        &self,
        target: &str,
        limit: i64,
    ) -> Result<Vec<HttpMetricRow>, sqlx::Error> {
        self.repository.fetch_http_metrics(target, limit).await
    }

    /// Get the latest ICMP metrics for a target
    pub async fn get_latest_icmp_metrics(
        &self,
        target: &str,
        limit: i64,
    ) -> Result<Vec<IcmpMetricRow>, sqlx::Error> {
        self.repository.fetch_icmp_metrics(target, limit).await
    }
}

impl Export for TimescaleExporter {
    #[allow(unreachable_patterns)]
    fn export(&self, probe_type: crate::ProbeType, data: ExporterRequest) -> Result<(), Error> {
        use crate::ProbeType;

        let pool = self.pool.clone();
        let labels = self.labels.clone();

        match probe_type {
            ProbeType::Http => {
                // HTTP metrics
                let up = data.metrics.get("up").copied().unwrap_or(0) as u8;
                let success = data.metrics.get("success").copied().unwrap_or(0) as u8;
                let dns_duration = data.metrics.get("dns_duration_ms").copied().unwrap_or(0) as u128;
                let status_code = data.metrics.get("status_code").copied().unwrap_or(0) as u16;
                let http_duration = data.metrics.get("http_duration_ms").copied().unwrap_or(0) as u128;
                let tls_duration = data.metrics.get("tls_duration_ms").map(|v| *v as u128);
                let tls_handshake = data.metrics.get("tls_handshake_ms").map(|v| *v as u128);
                let cert_expiration = data.metrics.get("cert_expiration_ts").map(|v| *v as i64);
                let cert_begin = data.metrics.get("cert_begin_ts").map(|v| *v as i64);

                tokio::spawn(async move {
                    let repository = TimescaleRepository::new(pool);
                    let labels_json = serde_json::to_value(&labels)
                        .unwrap_or(serde_json::Value::Null);
                    
                    let target = labels.get("target").map(|s| s.as_str()).unwrap_or("unknown");
                    let zone = labels.get("zone").map(|s| s.as_str());
                    let job = labels.get("job").map(|s| s.as_str());
                    let http_version = labels.get("http_version").map(|s| s.as_str());
                    let tls_version = labels.get("tls_version").map(|s| s.as_str());

                    if let Err(e) = repository.insert_http_metrics(
                        target,
                        zone,
                        job,
                        up as i16,
                        success as i16,
                        status_code as i32,
                        duration_to_i64(dns_duration),
                        duration_to_i64(http_duration),
                        tls_duration.map(duration_to_i64),
                        tls_handshake.map(duration_to_i64),
                        cert_expiration,
                        cert_begin,
                        http_version,
                        tls_version,
                        labels_json,
                    ).await {
                        log::error!("Failed to export HTTP metrics to TimescaleDB: {}", e);
                    }
                });

                Ok(())
            }
            ProbeType::Icmp => {
                // ICMP metrics
                let up = data.metrics.get("up").copied().unwrap_or(0) as u8;
                let rtt_ms = data.metrics.get("rtt_ms").copied().unwrap_or(0) as u128;

                tokio::spawn(async move {
                    let repository = TimescaleRepository::new(pool);
                    let labels_json = serde_json::to_value(&labels)
                        .unwrap_or(serde_json::Value::Null);
                    
                    let target = labels.get("target").map(|s| s.as_str()).unwrap_or("unknown");
                    let zone = labels.get("zone").map(|s| s.as_str());
                    let job = labels.get("job").map(|s| s.as_str());

                    if let Err(e) = repository.insert_icmp_metrics(
                        target,
                        zone,
                        job,
                        up as i16,
                        duration_to_i64(rtt_ms),
                        labels_json,
                    ).await {
                        log::error!("Failed to export ICMP metrics to TimescaleDB: {}", e);
                    }
                });

                Ok(())
            }
            _ => {
                Err(Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("TimescaleDB exporter does not support probe type: {}", probe_type)
                ))
            }
        }
    }
}
