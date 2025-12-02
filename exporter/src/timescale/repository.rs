use sqlx::PgPool;
use std::sync::Arc;

/// Repository for TimescaleDB database operations
///
/// Centralizes all SQL queries and database operations
pub struct TimescaleRepository {
    pool: Arc<PgPool>,
    schema: String,
}

impl TimescaleRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool, schema: "public".to_string() }
    }

    pub fn with_schema(pool: Arc<PgPool>, schema: String) -> Self {
        Self { pool, schema }
    }

    /// Create HTTP metrics table if it doesn't exist
    pub async fn create_http_metrics_table(&self) -> Result<(), sqlx::Error> {
        let query = format!(
            r#"
            CREATE TABLE IF NOT EXISTS {}.http_metrics (
                time TIMESTAMPTZ NOT NULL,
                target TEXT NOT NULL,
                zone TEXT,
                job TEXT,
                up SMALLINT NOT NULL,
                success SMALLINT NOT NULL,
                status_code INTEGER NOT NULL,
                dns_duration_ms BIGINT NOT NULL,
                http_duration_ms BIGINT NOT NULL,
                tls_duration_ms BIGINT,
                tls_handshake_ms BIGINT,
                cert_expiration_ts BIGINT,
                cert_begin_ts BIGINT,
                http_version TEXT,
                tls_version TEXT,
                labels JSONB
            )
            "#,
            self.schema
        );
        sqlx::query(&query).execute(&*self.pool).await?;

        Ok(())
    }

    /// Create ICMP metrics table if it doesn't exist
    pub async fn create_icmp_metrics_table(&self) -> Result<(), sqlx::Error> {
        let query = format!(
            r#"
            CREATE TABLE IF NOT EXISTS {}.icmp_metrics (
                time TIMESTAMPTZ NOT NULL,
                target TEXT NOT NULL,
                zone TEXT,
                job TEXT,
                up SMALLINT NOT NULL,
                rtt_ms BIGINT NOT NULL,
                labels JSONB
            )
            "#,
            self.schema
        );
        sqlx::query(&query).execute(&*self.pool).await?;

        Ok(())
    }

    /// Create hypertable for HTTP metrics
    pub async fn create_http_hypertable(&self) -> Result<(), sqlx::Error> {
        let query = format!(
            r#"
            SELECT create_hypertable('{}.http_metrics', 'time', 
                if_not_exists => TRUE,
                chunk_time_interval => INTERVAL '1 day'
            )
            "#,
            self.schema
        );
        let _ = sqlx::query(&query).execute(&*self.pool).await;

        Ok(())
    }

    /// Create hypertable for ICMP metrics
    pub async fn create_icmp_hypertable(&self) -> Result<(), sqlx::Error> {
        let query = format!(
            r#"
            SELECT create_hypertable('{}.icmp_metrics', 'time', 
                if_not_exists => TRUE,
                chunk_time_interval => INTERVAL '1 day'
            )
            "#,
            self.schema
        );
        let _ = sqlx::query(&query).execute(&*self.pool).await;

        Ok(())
    }

    /// Create index on HTTP metrics
    pub async fn create_http_metrics_index(&self) -> Result<(), sqlx::Error> {
        let query = format!(
            r#"
            CREATE INDEX IF NOT EXISTS idx_http_metrics_target_time 
            ON {}.http_metrics (target, time DESC)
            "#,
            self.schema
        );
        sqlx::query(&query).execute(&*self.pool).await?;

        Ok(())
    }

    /// Create index on ICMP metrics
    pub async fn create_icmp_metrics_index(&self) -> Result<(), sqlx::Error> {
        let query = format!(
            r#"
            CREATE INDEX IF NOT EXISTS idx_icmp_metrics_target_time 
            ON {}.icmp_metrics (target, time DESC)
            "#,
            self.schema
        );
        sqlx::query(&query).execute(&*self.pool).await?;

        Ok(())
    }

    /// Insert HTTP metrics
    pub async fn insert_http_metrics(&self, to_insert: HttpMetricRow) -> Result<(), sqlx::Error> {
        let query = format!(
            r#"
            INSERT INTO {}.http_metrics (
                time, target, zone, job, up, success, status_code,
                dns_duration_ms, http_duration_ms, tls_duration_ms, tls_handshake_ms,
                cert_expiration_ts, cert_begin_ts, http_version, tls_version, labels
            ) VALUES (NOW(), $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            "#,
            self.schema
        );
        sqlx::query(&query)
            .bind(to_insert.target)
            .bind(to_insert.zone)
            .bind(to_insert.job)
            .bind(to_insert.up)
            .bind(to_insert.success)
            .bind(to_insert.status_code)
            .bind(to_insert.dns_duration_ms)
            .bind(to_insert.http_duration_ms)
            .bind(to_insert.tls_duration_ms)
            .bind(to_insert.tls_handshake_ms)
            .bind(to_insert.cert_expiration_ts)
            .bind(to_insert.cert_begin_ts)
            .bind(to_insert.http_version)
            .bind(to_insert.tls_version)
            .bind(to_insert.labels)
            .execute(&*self.pool)
            .await?;

        Ok(())
    }

    /// Insert ICMP metrics
    pub async fn insert_icmp_metrics(&self, to_insert: IcmpMetricRow) -> Result<(), sqlx::Error> {
        let query = format!(
            r#"
            INSERT INTO {}.icmp_metrics (
                time, target, zone, job, up, rtt_ms, labels
            ) VALUES (NOW(), $1, $2, $3, $4, $5, $6)
            "#,
            self.schema
        );
        sqlx::query(&query)
            .bind(to_insert.target)
            .bind(to_insert.zone)
            .bind(to_insert.job)
            .bind(to_insert.up)
            .bind(to_insert.rtt_ms)
            .bind(to_insert.labels)
            .execute(&*self.pool)
            .await?;

        Ok(())
    }

    /// Fetch latest HTTP metrics for a target
    pub async fn fetch_http_metrics(
        &self,
        target: &str,
        limit: i64,
    ) -> Result<Vec<HttpMetricRow>, sqlx::Error> {
        let query = format!(
            r#"
            SELECT time, target, zone, job, up, success, status_code,
                   dns_duration_ms, http_duration_ms, tls_duration_ms, tls_handshake_ms,
                   cert_expiration_ts, cert_begin_ts, http_version, tls_version, labels
            FROM {}.http_metrics
            WHERE target = $1
            ORDER BY time DESC
            LIMIT $2
            "#,
            self.schema
        );
        let rows = sqlx::query_as::<_, HttpMetricRow>(&query)
            .bind(target)
            .bind(limit)
            .fetch_all(&*self.pool)
            .await?;

        Ok(rows)
    }

    /// Fetch latest ICMP metrics for a target
    pub async fn fetch_icmp_metrics(
        &self,
        target: &str,
        limit: i64,
    ) -> Result<Vec<IcmpMetricRow>, sqlx::Error> {
        let query = format!(
            r#"
            SELECT time, target, zone, job, up, rtt_ms, labels
            FROM {}.icmp_metrics
            WHERE target = $1
            ORDER BY time DESC
            LIMIT $2
            "#,
            self.schema
        );
        let rows = sqlx::query_as::<_, IcmpMetricRow>(&query)
            .bind(target)
            .bind(limit)
            .fetch_all(&*self.pool)
            .await?;

        Ok(rows)
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct HttpMetricRow {
    pub time: chrono::DateTime<chrono::Utc>,
    pub target: String,
    pub zone: Option<String>,
    pub job: Option<String>,
    pub up: i16,
    pub success: i16,
    pub status_code: i32,
    pub dns_duration_ms: i64,
    pub http_duration_ms: i64,
    pub tls_duration_ms: Option<i64>,
    pub tls_handshake_ms: Option<i64>,
    pub cert_expiration_ts: Option<i64>,
    pub cert_begin_ts: Option<i64>,
    pub http_version: Option<String>,
    pub tls_version: Option<String>,
    pub labels: Option<serde_json::Value>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct IcmpMetricRow {
    pub time: chrono::DateTime<chrono::Utc>,
    pub target: String,
    pub zone: Option<String>,
    pub job: Option<String>,
    pub up: i16,
    pub rtt_ms: i64,
    pub labels: Option<serde_json::Value>,
}
