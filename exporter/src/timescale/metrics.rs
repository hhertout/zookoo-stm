use crate::Exporter;
use crate::MetricData;
use crate::timescale::repository::{HttpMetricRow, IcmpMetricRow, TimescaleRepository};
use sqlx::PgPool;
use std::collections::HashMap;
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
        log::warn!(
            "Duration overflow detected: {} ms exceeds i64::MAX, clamping to maximum value",
            duration
        );
        i64::MAX
    })
}

/// TimescaleDB metrics exporter
///
/// Stores probe metrics in TimescaleDB hypertables for time-series analysis.
pub struct TimescaleExporter {
    pub pool: Arc<PgPool>,
    repository: TimescaleRepository,
    default_labels: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct HttpMetricsParams {
    pub up: u8,
    pub success: u8,
    pub dns_duration: u128,
    pub status_code: u16,
    pub http_duration: u128,
    pub tls_duration: Option<u128>,
    pub tls_handshake: Option<u128>,
    pub cert_expiration_ts: Option<i64>,
    pub cert_begin_ts: Option<i64>,
    pub target_labels: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct IcmpMetricsParams {
    pub up: u8,
    pub rtt_ms: u128,
    pub target_labels: HashMap<String, String>,
}

impl TimescaleExporter {
    /// Create a new TimescaleDB exporter with the given connection pool and labels
    /// Uses the default "public" schema
    pub fn new(pool: Arc<PgPool>, labels: HashMap<String, String>) -> Self {
        let repository = TimescaleRepository::new(pool.clone());
        Self { pool, repository, default_labels: labels }
    }

    /// Create a new TimescaleDB exporter with a custom schema
    pub fn with_schema(pool: Arc<PgPool>, labels: HashMap<String, String>, schema: String) -> Self {
        let repository = TimescaleRepository::with_schema(pool.clone(), schema);
        Self { pool, repository, default_labels: labels }
    }

    /// Merge default labels with target-specific labels (target labels take precedence)
    fn merge_labels(&self, target_labels: &HashMap<String, String>) -> HashMap<String, String> {
        let mut merged = self.default_labels.clone();
        for (key, value) in target_labels {
            merged.insert(key.clone(), value.clone());
        }
        merged
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

    pub async fn export_http_metrics(
        &self,
        params: HttpMetricsParams,
    ) -> Result<(), sqlx::Error> {
        let labels = self.merge_labels(&params.target_labels);
        let target = labels.get("target").map(|s| s.as_str()).unwrap_or("unknown");
        let zone = labels.get("zone").map(|s| s.as_str());
        let job = labels.get("job").map(|s| s.as_str());
        let http_version = labels.get("http_version").map(|s| s.as_str());
        let tls_version = labels.get("tls_version").map(|s| s.as_str());

        let labels_json = serde_json::to_value(&labels).unwrap_or(serde_json::Value::Null);

        let to_insert = HttpMetricRow {
            time: chrono::Utc::now(),
            target: target.to_string(),
            zone: zone.map(|s| s.to_string()),
            job: job.map(|s| s.to_string()),
            up: params.up as i16,
            success: params.success as i16,
            status_code: params.status_code as i32,
            dns_duration_ms: duration_to_i64(params.dns_duration),
            http_duration_ms: duration_to_i64(params.http_duration),
            tls_duration_ms: params.tls_duration.map(duration_to_i64),
            tls_handshake_ms: params.tls_handshake.map(duration_to_i64),
            cert_expiration_ts: params.cert_expiration_ts,
            cert_begin_ts: params.cert_begin_ts,
            http_version: http_version.map(|s| s.to_string()),
            tls_version: tls_version.map(|s| s.to_string()),
            labels: Some(labels_json.clone()),
        };
        self.repository.insert_http_metrics(to_insert).await
    }

    /// Export ICMP metrics to TimescaleDB
    pub async fn export_icmp_metrics(
        &self,
        params: IcmpMetricsParams,
    ) -> Result<(), sqlx::Error> {
        let labels = self.merge_labels(&params.target_labels);
        let target = labels.get("target").map(|s| s.as_str()).unwrap_or("unknown");
        let zone = labels.get("zone").map(|s| s.as_str());
        let job = labels.get("job").map(|s| s.as_str());

        let labels_json = serde_json::to_value(&labels).unwrap_or(serde_json::Value::Null);

        let to_insert = IcmpMetricRow {
            time: chrono::Utc::now(),
            target: target.to_string(),
            zone: zone.map(|s| s.to_string()),
            job: job.map(|s| s.to_string()),
            up: params.up as i16,
            rtt_ms: duration_to_i64(params.rtt_ms),
            labels: Some(labels_json.clone()),
        };

        self.repository.insert_icmp_metrics(to_insert).await
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

impl Exporter for TimescaleExporter {
    fn export(&self, probe_type: crate::ProbeType, metric_data: MetricData) {
        use crate::ProbeType;

        let pool = self.pool.clone();
        let metrics = metric_data.metrics;
        let labels = self.merge_labels(&metric_data.labels);

        match probe_type {
            ProbeType::Http => {
                // HTTP metrics
                let up = metrics.get("up").copied().unwrap_or(0) as u8;
                let success = metrics.get("success").copied().unwrap_or(0) as u8;
                let dns_duration = metrics.get("dns_duration_ms").copied().unwrap_or(0) as u128;
                let status_code = metrics.get("status_code").copied().unwrap_or(0) as u16;
                let http_duration = metrics.get("http_duration_ms").copied().unwrap_or(0) as u128;
                let tls_duration = metrics.get("tls_duration_ms").map(|v| *v as u128);
                let tls_handshake = metrics.get("tls_handshake_ms").map(|v| *v as u128);
                let cert_expiration = metrics.get("cert_expiration_ts").map(|v| *v as i64);
                let cert_begin = metrics.get("cert_begin_ts").map(|v| *v as i64);

                tokio::spawn(async move {
                    let repository = TimescaleRepository::new(pool);
                    let labels_json =
                        serde_json::to_value(&labels).unwrap_or(serde_json::Value::Null);

                    let target = labels.get("target").map(|s| s.as_str()).unwrap_or("unknown");
                    let zone = labels.get("zone").map(|s| s.as_str());
                    let job = labels.get("job").map(|s| s.as_str());
                    let http_version = labels.get("http_version").map(|s| s.as_str());
                    let tls_version = labels.get("tls_version").map(|s| s.as_str());

                    let to_insert = HttpMetricRow {
                        time: chrono::Utc::now(),
                        target: target.to_string(),
                        zone: zone.map(|s| s.to_string()),
                        job: job.map(|s| s.to_string()),
                        up: up as i16,
                        success: success as i16,
                        status_code: status_code as i32,
                        dns_duration_ms: duration_to_i64(dns_duration),
                        http_duration_ms: duration_to_i64(http_duration),
                        tls_duration_ms: tls_duration.map(duration_to_i64),
                        tls_handshake_ms: tls_handshake.map(duration_to_i64),
                        cert_expiration_ts: cert_expiration,
                        cert_begin_ts: cert_begin,
                        http_version: http_version.map(|s| s.to_string()),
                        tls_version: tls_version.map(|s| s.to_string()),
                        labels: Some(labels_json.clone()),
                    };
                    if let Err(e) = repository.insert_http_metrics(to_insert).await {
                        log::error!("Failed to export HTTP metrics to TimescaleDB: {}", e);
                    }
                });
            }
            ProbeType::Icmp => {
                // ICMP metrics
                let up = metrics.get("up").copied().unwrap_or(0) as u8;
                let rtt_ms = metrics.get("rtt_ms").copied().unwrap_or(0) as u128;

                tokio::spawn(async move {
                    let repository = TimescaleRepository::new(pool);
                    let labels_json =
                        serde_json::to_value(&labels).unwrap_or(serde_json::Value::Null);

                    let target = labels.get("target").map(|s| s.as_str()).unwrap_or("unknown");
                    let zone = labels.get("zone").map(|s| s.as_str());
                    let job = labels.get("job").map(|s| s.as_str());

                    let to_insert = IcmpMetricRow {
                        time: chrono::Utc::now(),
                        target: target.to_string(),
                        zone: zone.map(|s| s.to_string()),
                        job: job.map(|s| s.to_string()),
                        up: up as i16,
                        rtt_ms: duration_to_i64(rtt_ms),
                        labels: Some(labels_json.clone()),
                    };
                    if let Err(e) = repository.insert_icmp_metrics(to_insert).await {
                        log::error!("Failed to export ICMP metrics to TimescaleDB: {}", e);
                    }
                });
            }
        }
    }
}
