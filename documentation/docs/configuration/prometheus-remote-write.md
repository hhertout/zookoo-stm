# Prometheus Remote Write Exporter

This exporter sends metrics to Prometheus-compatible endpoints using the remote_write API format.

## Compatible Backends

- **Grafana Alloy** (`prometheus.receive_http` component)
- **Prometheus** (with remote_write receiver enabled)
- **Grafana Mimir**
- **Thanos**
- **Victoria Metrics**

## Format

- **Protocol**: Prometheus remote_write API (protobuf)
- **Compression**: Snappy
- **Endpoint**: `/api/v1/metrics/write`
- **Content-Type**: `application/x-protobuf`
- **Content-Encoding**: `snappy`

## Configuration

### Basic Configuration

```toml
[exporter.prometheus_remote_write]
url = "http://localhost:9999/api/v1/metrics/write"
job = "zookoo-stm"
instance = "my-instance-01"
```

### With Authentication

```toml
[exporter.prometheus_remote_write]
url = "https://prometheus.example.com/api/v1/metrics/write"
job = "zookoo-stm"
instance = "production-01"

[exporter.prometheus_remote_write.auth]
# Option 1: Basic Auth
username = "myuser"
password = "mypassword"

# Option 2: Bearer Token
# bearer = "your-token-here"
```

## Grafana Alloy Setup

### 1. Create Alloy Configuration

File: `alloy-config.alloy`

```hcl
// Receive metrics from zookoo via HTTP
prometheus.receive_http "zookoo" {
  http {
    listen_address = "0.0.0.0"
    listen_port = 9999
  }
  
  forward_to = [prometheus.remote_write.backend.receiver]
}

// Forward to Prometheus/Mimir backend
prometheus.remote_write "backend" {
  endpoint {
    url = "http://localhost:9009/api/v1/push"
    
    // Optional authentication
    // basic_auth {
    //   username = "admin"
    //   password = "password"
    // }
  }
  
  queue_config {
    capacity = 10000
    max_shards = 10
    max_samples_per_send = 1000
  }
}

// Optional: Self-monitoring
prometheus.exporter.self "alloy" { }

prometheus.scrape "alloy" {
  targets = prometheus.exporter.self.alloy.targets
  forward_to = [prometheus.remote_write.backend.receiver]
}
```

### 2. Start Alloy

```bash
# Install Alloy (if not already installed)
# macOS
brew install grafana/grafana/alloy

# Start Alloy
alloy run alloy-config.alloy
```

### 3. Configure zookoo

```toml
[exporter.prometheus_remote_write]
url = "http://localhost:9999/api/v1/metrics/write"
job = "zookoo-stm"
instance = "test-instance"
```

### 4. Run zookoo

```bash
cargo run --release -- --config test-alloy-config.toml
```

## Testing

### Quick Test Script

```bash
./scripts/test_alloy.sh
```

### Manual Testing

1. **Start Alloy**:
```bash
alloy run dev/alloy-zookoo.alloy
```

2. **Verify Alloy is listening**:
```bash
curl http://localhost:9999
```

3. **Run zookoo**:
```bash
cargo run -- --config test-alloy-config.toml
```

4. **Check metrics are being sent**:
- Alloy logs should show incoming requests
- Backend (Prometheus/Mimir) should receive metrics

## Metrics Sent

The exporter automatically adds these labels to all metrics:

- `job`: From configuration (default: "zookoo-stm")
- `instance`: From configuration (optional)
- `zone`: From probe configuration (if set)
- All custom labels from targets

### Example Metrics

```
http_probe_success{job="zookoo-stm",instance="test",target="https://example.com",status="200"} 1.0
http_probe_duration_seconds{job="zookoo-stm",instance="test",target="https://example.com"} 0.234
icmp_probe_success{job="zookoo-stm",instance="test",address="8.8.8.8"} 1.0
icmp_probe_rtt_seconds{job="zookoo-stm",instance="test",address="8.8.8.8"} 0.012
```

## Architecture

```
┌─────────┐  remote_write   ┌───────────────┐  remote_write   ┌────────────┐
│ zookoo  │ ──────────────> │ Grafana Alloy │ ──────────────> │ Prometheus │
│         │  protobuf+snappy│               │                 │   /Mimir   │
└─────────┘                 └───────────────┘                 └────────────┘
                                    │
                                    │ forward_to
                                    v
                            ┌───────────────┐
                            │   Grafana     │
                            │  Dashboards   │
                            └───────────────┘
```

## Performance Considerations

### Advantages over Pushgateway

1. **Better Performance**: Binary protocol (protobuf) + Snappy compression
2. **Lower Network Usage**: ~40-60% compression ratio
3. **Streaming**: Continuous data flow vs batch push
4. **Native Protocol**: Standard Prometheus format
5. **Scalability**: Designed for high-throughput scenarios

### Batch Size

The exporter can send multiple metrics in a single request:

```rust
let metrics = vec![
    ("http_requests_total".to_string(), 1234.0, labels1),
    ("http_duration_seconds".to_string(), 0.456, labels2),
];
exporter.push_metrics(metrics, None).await?;
```

## Troubleshooting

### Connection Refused

**Problem**: `Failed to send remote_write: Connection refused`

**Solution**:
1. Verify Alloy is running: `curl http://localhost:9999`
2. Check Alloy logs for errors
3. Verify firewall rules

### Authentication Failed

**Problem**: `Remote write failed with status 401`

**Solution**:
1. Verify credentials in configuration
2. Check if bearer token or basic auth is required
3. Ensure auth section matches backend requirements

### Metrics Not Appearing

**Problem**: Metrics sent but not visible in Grafana

**Solution**:
1. Check Alloy logs: `alloy run --config.file=alloy-config.alloy`
2. Verify `forward_to` configuration in Alloy
3. Check backend (Prometheus/Mimir) is receiving data
4. Query Prometheus directly: `curl http://localhost:9090/api/v1/query?query=zookoo`

### Compression Errors

**Problem**: `Snappy compression failed`

**Solution**:
- This is rare; usually indicates corrupted data
- Verify protobuf encoding is correct
- Check for memory issues

## Advanced Configuration

### Custom Labels

Add extra labels to all metrics:

```toml
[default]
probe_zone = "us-east-1"  # Automatically added as 'zone' label

[[targets.http]]
name = "api"
url = "https://api.example.com"

[[targets.http.labels]]
environment = "production"
team = "platform"
```

### Multiple Exporters

You can enable both Pushgateway and remote_write:

```toml
# For legacy systems
[exporter.prometheus]
url = "http://pushgateway:9091"
job = "zookoo-stm"

# For modern systems (Alloy)
[exporter.prometheus_remote_write]
url = "http://alloy:9999/api/v1/metrics/write"
job = "zookoo-stm"
```

## API Reference

### Configuration Structure

```rust
pub struct PrometheusRemoteWriteConfig {
    pub url: String,                          // Required: endpoint URL
    pub job: String,                          // Required: job label
    pub instance: Option<String>,             // Optional: instance label
    pub auth: Option<AuthConfiguration>,      // Optional: authentication
    pub extra_labels: HashMap<String, String>, // Optional: additional labels
}
```

### Methods

```rust
// Create exporter
let exporter = PrometheusRemoteWrite::new(config)?;

// Push single metric
exporter.push_metric(
    "http_requests_total",
    1234.0,
    labels,
    Some(timestamp_ms)
).await?;

// Push multiple metrics (more efficient)
exporter.push_metrics(vec![
    (name1, value1, labels1),
    (name2, value2, labels2),
], Some(timestamp_ms)).await?;
```

## Resources

- [Prometheus Remote Write Spec](https://prometheus.io/docs/prometheus/latest/querying/api/#remote-write-receiver)
- [Grafana Alloy Documentation](https://grafana.com/docs/alloy/latest/)
- [prometheus.receive_http Component](https://grafana.com/docs/alloy/latest/reference/components/prometheus/prometheus.receive_http/)
- [Grafana Mimir](https://grafana.com/oss/mimir/)
