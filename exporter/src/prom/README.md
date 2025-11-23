# Prometheus Pushgateway Exporter

## Overview

This module provides a Prometheus Pushgateway exporter for pushing metrics to Prometheus via the Pushgateway API.

## Features

- ✅ Push metrics to Prometheus Pushgateway
- ✅ Support for all Prometheus metric types (counter, gauge, histogram, summary, untyped)
- ✅ Custom labels and grouping
- ✅ Authentication (Basic Auth & Bearer Token)
- ✅ Metric deletion support
- ✅ Proper label escaping
- ✅ Timestamp support

## Configuration

```rust
use std::collections::HashMap;
use exporter::prometheus::{PrometheusPushgatewayConfig, PrometheusPushgateway};
use exporter::config::AuthConfiguration;

let config = PrometheusPushgatewayConfig {
    url: "http://localhost:9091".to_string(),
    job: "zookoo_prober".to_string(),
    instance: Some("server1".to_string()),
    auth: Some(AuthConfiguration {
        username: Some("admin".to_string()),
        password: Some("secret".to_string()),
        bearer: None,
    }),
    grouping_labels: HashMap::new(),
};

let exporter = PrometheusPushgateway::new(config)?;
```

## Usage Examples

### Simple Gauge Metric

```rust
use std::collections::HashMap;
use exporter::prometheus::{create_gauge_metric, PrometheusPushgateway};

let mut labels = HashMap::new();
labels.insert("target".to_string(), "example.com".to_string());
labels.insert("status_code".to_string(), "200".to_string());

let metric = create_gauge_metric(
    "http_request_duration_seconds",
    "HTTP request duration in seconds",
    0.245,
    labels,
);

exporter.push_metrics(vec![metric]).await?;
```

### Counter Metric

```rust
use exporter::prometheus::create_counter_metric;

let metric = create_counter_metric(
    "http_requests_total",
    "Total number of HTTP requests",
    1500.0,
    labels,
);

exporter.push_metrics(vec![metric]).await?;
```

### Multiple Metrics

```rust
let metrics = vec![
    create_gauge_metric(
        "probe_success",
        "Probe success status",
        1.0,
        labels.clone(),
    ),
    create_gauge_metric(
        "probe_duration_seconds",
        "Probe duration in seconds",
        0.123,
        labels.clone(),
    ),
    create_counter_metric(
        "probe_total",
        "Total number of probes",
        42.0,
        labels,
    ),
];

exporter.push_metrics(metrics).await?;
```

### Custom Metric with Timestamp

```rust
use std::time::SystemTime;
use exporter::prometheus::{PrometheusMetric, MetricType};

let timestamp = SystemTime::now()
    .duration_since(SystemTime::UNIX_EPOCH)?
    .as_millis() as i64;

let metric = PrometheusMetric {
    name: "custom_metric".to_string(),
    metric_type: MetricType::Gauge,
    help: "Custom metric with timestamp".to_string(),
    value: 42.0,
    labels: labels,
    timestamp: Some(timestamp),
};

exporter.push_metrics(vec![metric]).await?;
```

### Delete Metrics

```rust
// Delete all metrics for this job/instance
exporter.delete_metrics().await?;
```

## Prometheus Exposition Format

The exporter generates metrics in the Prometheus text exposition format:

```
# HELP http_request_duration_seconds HTTP request duration in seconds
# TYPE http_request_duration_seconds gauge
http_request_duration_seconds{target="example.com",status="200"} 0.245

# HELP probe_success Probe success status
# TYPE probe_success gauge
probe_success{target="example.com"} 1
```

## Grouping Labels

Grouping labels are used to organize metrics in the Pushgateway:

```rust
let mut grouping_labels = HashMap::new();
grouping_labels.insert("env".to_string(), "production".to_string());
grouping_labels.insert("region".to_string(), "us-east-1".to_string());

let config = PrometheusPushgatewayConfig {
    url: "http://pushgateway:9091".to_string(),
    job: "zookoo_prober".to_string(),
    instance: Some("server1".to_string()),
    auth: None,
    grouping_labels,
};
```

This creates a URL like:
```
http://pushgateway:9091/metrics/job/zookoo_prober/instance/server1/env/production/region/us-east-1
```

## Authentication

### Basic Auth

```rust
let auth = AuthConfiguration {
    username: Some("admin".to_string()),
    password: Some("secret".to_string()),
    bearer: None,
};
```

### Bearer Token

```rust
let auth = AuthConfiguration {
    username: None,
    password: None,
    bearer: Some("my-token-123".to_string()),
};
```

## Integration with Configuration

In your `config.toml`:

```toml
[exporter.prometheus]
url = "http://localhost:9091"
job = "zookoo_prober"
instance = "server1"

[exporter.prometheus.auth]
username = "admin"
password = "secret"
```

## Error Handling

```rust
match exporter.push_metrics(metrics).await {
    Ok(_) => log::info!("Metrics pushed successfully"),
    Err(e) => log::error!("Failed to push metrics: {}", e),
}
```

## Best Practices

1. **Use descriptive metric names**: Follow Prometheus naming conventions
   - `http_request_duration_seconds` (not `httpRequestDuration`)
   - Use base units (seconds, bytes, etc.)

2. **Add relevant labels**: Make metrics queryable
   ```rust
   labels.insert("target".to_string(), "example.com".to_string());
   labels.insert("method".to_string(), "GET".to_string());
   labels.insert("status".to_string(), "200".to_string());
   ```

3. **Use appropriate metric types**:
   - **Counter**: Values that only increase (requests_total)
   - **Gauge**: Values that can go up/down (temperature, cpu_usage)
   - **Histogram**: Distributions (request_duration_seconds)

4. **Batch metrics**: Push multiple metrics together for efficiency

5. **Handle errors gracefully**: Don't let export failures break your application

## Testing

Run tests:
```bash
cargo test --package exporter prometheus
```

All tests pass: ✅
- `test_format_metrics` - Validates metric formatting
- `test_build_url` - Validates URL construction
- `test_escape_label_value` - Validates label escaping
- `test_create_gauge_metric` - Validates gauge creation
- `test_create_counter_metric` - Validates counter creation
