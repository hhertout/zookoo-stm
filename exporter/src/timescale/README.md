# TimescaleDB Exporter

The TimescaleDB exporter allows Zookoo to store probe metrics in a PostgreSQL database with the TimescaleDB extension. This provides powerful time-series data storage and querying capabilities.

## Features

- **Time-Series Optimized**: Uses TimescaleDB hypertables for efficient time-series data storage
- **Automatic Schema Management**: Creates tables and hypertables automatically on startup
- **Separate Tables**: HTTP and ICMP metrics stored in separate optimized tables
- **Rich Metadata**: Stores all probe labels, TLS information, and certificate details
- **Fast Queries**: Automatic indexing on target and time columns
- **JSON Labels**: All probe labels stored as JSONB for flexible querying

## Configuration

Add the TimescaleDB exporter to your `config.toml`:

```toml
[exporters.timescale]
connection_string = "postgresql://user:password@localhost:5432/database"
# Optional: Specify database schema (default: "public")
schema = "monitoring"
```

### Configuration Parameters

- **`connection_string`** (required): PostgreSQL connection URL
- **`schema`** (optional): Database schema name. Defaults to `"public"` if not specified.

Using a custom schema allows you to:
- Isolate metrics from other database objects
- Apply schema-level permissions
- Organize multiple environments (dev, staging, prod) in the same database
- Follow organizational naming conventions

### Connection String Format

```
postgresql://[user[:password]@][host][:port][/dbname][?param1=value1&...]
```

Examples:
```toml
# Local development (default schema)
connection_string = "postgresql://zookoo:zookoo@localhost:5432/zookoo"

# Production with SSL and custom schema
connection_string = "postgresql://user:pass@timescaledb.example.com:5432/metrics?sslmode=require"
schema = "production"

# Using custom schema for isolation
connection_string = "postgresql://zookoo:zookoo@timescaledb:5432/zookoo"
schema = "monitoring"
```

## Database Schema

All tables are created in the configured schema (default: `public`). The examples below show `public` schema, but you can use any schema name in your configuration.

### HTTP Metrics Table

The exporter automatically creates the `http_metrics` hypertable:

```sql
CREATE TABLE public.http_metrics (
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
);

SELECT create_hypertable('http_metrics', 'time', 
    chunk_time_interval => INTERVAL '1 day'
);

CREATE INDEX idx_http_metrics_target_time 
ON http_metrics (target, time DESC);
```

### ICMP Metrics Table

```sql
CREATE TABLE icmp_metrics (
    time TIMESTAMPTZ NOT NULL,
    target TEXT NOT NULL,
    zone TEXT,
    job TEXT,
    up SMALLINT NOT NULL,
    rtt_ms BIGINT NOT NULL,
    labels JSONB
);

SELECT create_hypertable('icmp_metrics', 'time', 
    chunk_time_interval => INTERVAL '1 day'
);

CREATE INDEX idx_icmp_metrics_target_time 
ON icmp_metrics (target, time DESC);
```

## Data Types and Limits

### Duration Storage

Duration metrics (DNS resolution, HTTP request, TLS handshake, ICMP RTT) are stored as `BIGINT` (signed 64-bit integer) representing **milliseconds**.

**Theoretical Maximum Duration**: 
- `i64::MAX` = 9,223,372,036,854,775,807 milliseconds
- ≈ 292,471,208 years
- ≈ 106,751,991,167,300 days
- ≈ 2,562,047,788,015 hours

**Practical Application**:
For network probes with typical timeouts of 1-120 seconds, this storage format is more than sufficient. The exporter includes overflow protection that clamps any duration exceeding `i64::MAX` to the maximum representable value (extremely unlikely in practice).

**Type Conversion Safety**:
The internal `duration_to_i64()` function uses checked casts to prevent data loss:
- Normal values: Direct conversion from u128 to i64
- Overflow values: Clamped to i64::MAX with warning log
- No panics: Graceful handling of edge cases

### Integer Ranges

| Column | Type | Range | Purpose |
|--------|------|-------|---------|
| `up` | SMALLINT | 0-1 | Target reachability (binary flag) |
| `success` | SMALLINT | 0-1 | Probe success (binary flag) |
| `status_code` | INTEGER | 100-599 | HTTP status code |
| `*_duration_ms` | BIGINT | 0 to 2^63-1 | Millisecond durations |
| `*_ts` | BIGINT | Unix timestamp | Certificate validity timestamps |

### JSON Labels

The `labels` column uses PostgreSQL's `JSONB` format for efficient storage and querying of arbitrary key-value metadata. All probe labels (target, zone, job, etc.) are preserved in this field.

## Querying Data

### Recent HTTP Metrics

```sql
-- Get last 10 HTTP checks for a target
SELECT time, target, status_code, http_duration_ms, up
FROM http_metrics
WHERE target = 'https://example.com'
ORDER BY time DESC
LIMIT 10;
```

### Average Response Times

```sql
-- Average HTTP duration over last hour
SELECT 
    target,
    AVG(http_duration_ms) as avg_duration,
    MAX(http_duration_ms) as max_duration,
    MIN(http_duration_ms) as min_duration
FROM http_metrics
WHERE time > NOW() - INTERVAL '1 hour'
GROUP BY target;
```

### Success Rate

```sql
-- Success rate by target over last 24 hours
SELECT 
    target,
    COUNT(*) as total_checks,
    SUM(success) as successful_checks,
    (SUM(success)::float / COUNT(*)::float * 100) as success_rate
FROM http_metrics
WHERE time > NOW() - INTERVAL '24 hours'
GROUP BY target
ORDER BY success_rate ASC;
```

### TLS Certificate Expiration

```sql
-- Find certificates expiring soon (within 30 days)
SELECT DISTINCT ON (target)
    target,
    cert_expiration_ts,
    to_timestamp(cert_expiration_ts) as expiration_date,
    (to_timestamp(cert_expiration_ts) - NOW()) as time_until_expiration
FROM http_metrics
WHERE cert_expiration_ts IS NOT NULL
  AND to_timestamp(cert_expiration_ts) < NOW() + INTERVAL '30 days'
ORDER BY target, time DESC;
```

### ICMP Round-Trip Times

```sql
-- Average RTT by target over last hour
SELECT 
    target,
    AVG(rtt_ms) as avg_rtt,
    MAX(rtt_ms) as max_rtt,
    MIN(rtt_ms) as min_rtt
FROM icmp_metrics
WHERE time > NOW() - INTERVAL '1 hour'
GROUP BY target;
```

### Time-Series Downsampling

```sql
-- 5-minute averages of HTTP duration
SELECT 
    time_bucket('5 minutes', time) as bucket,
    target,
    AVG(http_duration_ms) as avg_duration,
    COUNT(*) as num_checks
FROM http_metrics
WHERE time > NOW() - INTERVAL '1 day'
GROUP BY bucket, target
ORDER BY bucket DESC, target;
```

## Custom Schema Setup

### Creating a Custom Schema

To use a custom schema instead of `public`:

1. **Create the schema in PostgreSQL**:
```sql
CREATE SCHEMA IF NOT EXISTS monitoring;
GRANT USAGE ON SCHEMA monitoring TO zookoo;
GRANT CREATE ON SCHEMA monitoring TO zookoo;
```

2. **Configure Zookoo**:
```toml
[exporter.timescale]
connection_string = "postgresql://zookoo:zookoo@localhost:5432/zookoo"
schema = "monitoring"
```

3. **Start Zookoo**: Tables and hypertables will be created automatically in the specified schema.

### Benefits of Custom Schemas

- **Isolation**: Separate monitoring data from application tables
- **Permissions**: Apply schema-level access control
- **Multi-tenant**: Run multiple environments in one database
- **Organization**: Follow naming conventions (e.g., `prod`, `staging`, `monitoring`)

### Querying Custom Schema

When querying metrics in a custom schema, either:

**Option 1: Schema-qualified queries**
```sql
SELECT * FROM monitoring.http_metrics WHERE target = 'https://example.com';
```

**Option 2: Set search_path**
```sql
SET search_path TO monitoring, public;
SELECT * FROM http_metrics WHERE target = 'https://example.com';
```

## Docker Deployment

Use the provided Docker Compose stack for easy local testing:

```bash
cd dev
docker-compose -f docker-compose-timescale.yml up -d
```

This starts:
- **TimescaleDB**: PostgreSQL 17 with TimescaleDB extension on port 5432
- **Zookoo**: Configured to export metrics to TimescaleDB

### Access the Database

```bash
# Using psql
docker exec -it zookoo-timescaledb psql -U zookoo -d zookoo

# View recent metrics
docker exec -it zookoo-timescaledb psql -U zookoo -d zookoo \
  -c "SELECT time, target, status_code, http_duration_ms FROM http_metrics ORDER BY time DESC LIMIT 10;"
```

## TimescaleDB Features

### Automatic Data Retention

Set up automatic data retention policies:

```sql
-- Keep 90 days of HTTP metrics
SELECT add_retention_policy('http_metrics', INTERVAL '90 days');

-- Keep 30 days of ICMP metrics
SELECT add_retention_policy('icmp_metrics', INTERVAL '30 days');
```

### Compression

Enable automatic compression for older data:

```sql
-- Compress HTTP metrics older than 7 days
ALTER TABLE http_metrics SET (
    timescaledb.compress,
    timescaledb.compress_segmentby = 'target'
);

SELECT add_compression_policy('http_metrics', INTERVAL '7 days');

-- Compress ICMP metrics older than 7 days
ALTER TABLE icmp_metrics SET (
    timescaledb.compress,
    timescaledb.compress_segmentby = 'target'
);

SELECT add_compression_policy('icmp_metrics', INTERVAL '7 days');
```

### Continuous Aggregates

Create materialized views for faster queries:

```sql
-- Hourly rollup of HTTP metrics
CREATE MATERIALIZED VIEW http_metrics_hourly
WITH (timescaledb.continuous) AS
SELECT 
    time_bucket('1 hour', time) as bucket,
    target,
    AVG(http_duration_ms) as avg_duration,
    MAX(http_duration_ms) as max_duration,
    MIN(http_duration_ms) as min_duration,
    AVG(dns_duration_ms) as avg_dns_duration,
    SUM(success) / COUNT(*) as success_rate,
    COUNT(*) as num_checks
FROM http_metrics
GROUP BY bucket, target;

-- Refresh policy (automatically)
SELECT add_continuous_aggregate_policy('http_metrics_hourly',
    start_offset => INTERVAL '2 hours',
    end_offset => INTERVAL '1 hour',
    schedule_interval => INTERVAL '1 hour'
);
```

## Integration with Grafana

TimescaleDB works seamlessly with Grafana's PostgreSQL data source:

1. Add PostgreSQL data source in Grafana
2. Configure connection:
   - **Host**: `timescaledb:5432` (or your host)
   - **Database**: `zookoo`
   - **User**: `zookoo`
   - **Password**: `zookoo`
   - **SSL Mode**: Disable (for local dev)

3. Create queries using the PostgreSQL query builder or raw SQL

### Example Grafana Query

```sql
SELECT 
    time as "time",
    target,
    http_duration_ms as "Duration (ms)"
FROM http_metrics
WHERE 
    time BETWEEN $__timeFrom() AND $__timeTo()
    AND target = '$target'
ORDER BY time
```

## Performance Considerations

- **Chunk Interval**: Default 1 day chunks work well for most use cases
- **Indexes**: Automatically created on `(target, time)` for fast filtering
- **Connection Pooling**: sqlx handles connection pooling automatically
- **Batch Inserts**: Each metric is inserted asynchronously via tokio::spawn
- **Compression**: Use TimescaleDB compression for data older than 7 days
- **Retention**: Set up retention policies to automatically drop old data

## Troubleshooting

### Connection Errors

```bash
# Check TimescaleDB is running
docker ps | grep timescaledb

# Check logs
docker logs zookoo-timescaledb

# Test connection
docker exec -it zookoo-timescaledb psql -U zookoo -d zookoo -c "SELECT version();"
```

### Schema Issues

If tables don't exist:

```bash
# Check if extension is enabled
docker exec -it zookoo-timescaledb psql -U zookoo -d zookoo \
  -c "SELECT * FROM pg_extension WHERE extname = 'timescaledb';"

# Manually run schema initialization (if needed)
# Tables are created automatically when Zookoo starts
```

### Query Performance

```sql
-- Check hypertable info
SELECT * FROM timescaledb_information.hypertables;

-- Check chunk info
SELECT * FROM timescaledb_information.chunks WHERE hypertable_name = 'http_metrics';

-- Analyze table statistics
ANALYZE http_metrics;
ANALYZE icmp_metrics;
```

## Comparison with Other Exporters

| Feature | TimescaleDB | Prometheus | OTLP |
|---------|-------------|------------|------|
| Time-Series Storage | ✅ Native | ✅ Native | ❌ |
| SQL Queries | ✅ Full SQL | ❌ PromQL only | ❌ |
| Data Retention | ✅ Flexible policies | ✅ Manual | ❌ |
| Compression | ✅ Automatic | ✅ Built-in | ❌ |
| Relational Data | ✅ Full support | ❌ Labels only | ❌ |
| Alerting | ⚠️ Via Grafana | ✅ Native | ⚠️ Via backend |
| Scalability | ✅ Horizontal | ✅ Federation | ✅ High |

## Best Practices

1. **Set Retention Policies**: Automatically drop old data
2. **Enable Compression**: Save storage space for historical data
3. **Use Continuous Aggregates**: Pre-compute common rollups
4. **Index Wisely**: Default indexes cover most queries
5. **Monitor Connection Pool**: Check sqlx metrics
6. **Regular VACUUM**: PostgreSQL maintenance for optimal performance

## Example Complete Configuration

```toml
# config.toml
[global]
probe_location = "us-east-1"

[http.targets]
url = "https://example.com"
name = "example"
zone = "production"

[icmp.targets]
target = "8.8.8.8"
name = "google-dns"
zone = "external"

[exporters.timescale]
connection_string = "postgresql://zookoo:zookoo@timescaledb:5432/zookoo"
```

## References

- [TimescaleDB Documentation](https://docs.timescale.com/)
- [PostgreSQL Documentation](https://www.postgresql.org/docs/)
- [sqlx Documentation](https://github.com/launchbadge/sqlx)
