use crate::timescale::repository::{TimescaleRepository, HttpMetricRow, IcmpMetricRow};
use crate::{Export, ExporterRequest};
use sqlx::PgPool;
use std::collections::HashMap;
use std::io::Error;
use std::sync::Arc;

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
    pub fn new(pool: Arc<PgPool>, labels: HashMap<String, String>) -> Self {
        let repository = TimescaleRepository::new(pool.clone());
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
            dns_duration as i64,
            http_duration as i64,
            tls_duration.map(|d| d as i64),
            tls_handshake.map(|d| d as i64),
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
            rtt_ms as i64,
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
                        dns_duration as i64,
                        http_duration as i64,
                        tls_duration.map(|d| d as i64),
                        tls_handshake.map(|d| d as i64),
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
                        rtt_ms as i64,
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
