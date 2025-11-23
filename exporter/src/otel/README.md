# OpenTelemetry (OTLP) Exporter

This module provides OpenTelemetry Protocol (OTLP) metric export capabilities for Zookoo, allowing you to send probe metrics to any OTLP-compatible backend such as Grafana Alloy, OpenTelemetry Collector, Jaeger, or other observability platforms.

## Overview

The OTLP exporter uses the official OpenTelemetry SDK to export metrics in the standard OTLP format. It supports both gRPC and HTTP transports with full histogram support for latency metrics.

## Features

- ✅ **OTLP gRPC** - Native gRPC transport (recommended)
- ✅ **OTLP HTTP** - HTTP/protobuf transport
- ✅ **Histograms** - Latency distributions for DNS, HTTP, and TLS metrics
- ✅ **Counters** - Request totals and success rates
- ✅ **Gauges** - Certificate expiration timestamps
- ✅ **Resource Attributes** - Service metadata and labels
- ✅ **TLS Support** - Configurable TLS with custom certificates
- ✅ **Authentication** - Basic auth and bearer token support

## Configuration

### Basic Configuration

```toml
[exporter.otel]
url = "http://otel-collector:4317"
tls_insecure = true

[exporter.metrics]
enpoint = "http://otel-collector:4317"
service_name = "zookoo"
service_namespace = "monitoring"
service_version = "1.0.0"
service_instance_id = "zookoo-prod-01"
```

### With TLS

```toml
[exporter.otel]
url = "https://otel-collector:4317"
tls_insecure = false
cert_path = "/path/to/ca-cert.pem"
```

### With Authentication

```toml
[exporter.otel]
url = "https://otel-collector:4317"
tls_insecure = false

[exporter.otel.auth]
username = "admin"
password = "secret"
# or bearer token
bearer = "your-token-here"
```

## Exported Metrics

### HTTP/HTTPS Probe Metrics

| Metric Name | Type | Description | Labels |
|------------|------|-------------|--------|
| `probe_http_request_total` | Counter | Total number of HTTP requests | target, status_code, http_version, zone, job |
| `probe_http_request_duration_milliseconds` | Histogram | Total HTTP request duration | target, status_code, zone, job |
| `probe_dns_lookup_duration_milliseconds` | Histogram | DNS resolution time | target, zone, job |
| `probe_tls_duration_milliseconds` | Histogram | TLS handshake duration (HTTPS only) | target, tls_version, zone, job |
| `probe_tls_handshake_duration_milliseconds` | Histogram | TLS negotiation time (HTTPS only) | target, tls_version, zone, job |
| `probe_cert_expiration_ts` | Gauge | Certificate expiration timestamp (HTTPS only) | target, issuer, subject, algo, zone, job |
| `probe_cert_begin_ts` | Gauge | Certificate validity start timestamp (HTTPS only) | target, issuer, subject, algo, zone, job |

### ICMP Probe Metrics

| Metric Name | Type | Description | Labels |
|------------|------|-------------|--------|
| `probe_icmp_request_total` | Counter | Total number of ICMP requests | target, zone, job |
| `probe_icmp_rtt_milliseconds` | Histogram | ICMP round-trip time | target, zone, job |

### Histogram Buckets

The exporter uses explicit bucket boundaries optimized for network latency measurements:

- **DNS & HTTP**: `[10, 50, 100, 250, 500, 1000, 2500, 5000, 10000]` milliseconds
- **TLS Handshake**: `[50, 100, 250, 500, 1000, 2500, 5000]` milliseconds
- **ICMP RTT**: `[1, 5, 10, 25, 50, 100, 250, 500]` milliseconds

## Integration Examples

### Grafana Alloy

Use Grafana Alloy to receive OTLP metrics and forward to Prometheus:

**Alloy Config (`alloy-config.alloy`):**

```hcl
otelcol.receiver.otlp "zookoo" {
  grpc {
    endpoint = "0.0.0.0:4317"
  }

  http {
    endpoint = "0.0.0.0:4318"
  }

  output {
    metrics = [otelcol.processor.batch.default.input]
  }
}

otelcol.processor.batch "default" {
  timeout          = "10s"
  send_batch_size  = 1024

  output {
    metrics = [otelcol.exporter.prometheus.default.input]
  }
}

otelcol.exporter.prometheus "default" {
  forward_to = [prometheus.remote_write.prom.receiver]
}

prometheus.remote_write "prom" {
  endpoint {
    url = "http://prometheus:9090/api/v1/write"
  }
}
```

**Zookoo Config:**

```toml
[exporter.otel]
url = "http://alloy:4317"
tls_insecure = true

[exporter.metrics]
enpoint = "http://alloy:4317"
service_name = "zookoo"
service_namespace = "monitoring"
service_version = "1.0.0"
service_instance_id = "zookoo-prod-01"
```

**Docker Compose:**

```yaml
version: '3.8'

services:
  alloy:
    image: grafana/alloy:latest
    ports:
      - "4317:4317"  # OTLP gRPC
      - "4318:4318"  # OTLP HTTP
    volumes:
      - ./alloy-config.alloy:/etc/alloy/config.alloy
    command:
      - run
      - --server.http.listen-addr=0.0.0.0:12345
      - /etc/alloy/config.alloy

  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    command:
      - --web.enable-remote-write-receiver
      - --config.file=/etc/prometheus/prometheus.yml
```

### OpenTelemetry Collector

**Collector Config (`otel-collector-config.yaml`):**

```yaml
receivers:
  otlp:
    protocols:
      grpc:
        endpoint: 0.0.0.0:4317
      http:
        endpoint: 0.0.0.0:4318

processors:
  batch:
    timeout: 10s
    send_batch_size: 1024

exporters:
  prometheusremotewrite:
    endpoint: http://prometheus:9090/api/v1/write
    
  logging:
    loglevel: info

service:
  pipelines:
    metrics:
      receivers: [otlp]
      processors: [batch]
      exporters: [prometheusremotewrite, logging]
```

**Zookoo Config:**

```toml
[exporter.otel]
url = "http://otel-collector:4317"
tls_insecure = true

[exporter.metrics]
enpoint = "http://otel-collector:4317"
service_name = "zookoo"
service_namespace = "monitoring"
service_version = "1.0.0"
service_instance_id = "zookoo-collector"
```

### Grafana Cloud

```toml
[exporter.otel]
url = "https://otlp-gateway-prod-us-central-0.grafana.net:443"
tls_insecure = false

[exporter.otel.auth]
username = "your-instance-id"
password = "your-api-token"

[exporter.metrics]
enpoint = "https://otlp-gateway-prod-us-central-0.grafana.net:443"
service_name = "zookoo"
service_namespace = "monitoring"
service_version = "1.0.0"
service_instance_id = "zookoo-prod"
```

## Architecture

### Module Structure

```
exporter/src/otel/
├── mod.rs              # Module exports
└── metrics.rs          # MetricsExporter implementation
```

### Key Components

**`MetricsExporter`** - High-level metrics exporter with OpenTelemetry SDK integration

```rust
pub struct MetricsExporter {
    labels: HashMap<String, String>,
    meter: Meter,
}

impl MetricsExporter {
    pub fn new(labels: HashMap<String, String>) -> Self
    pub fn export_metrics(&self, ...) // HTTP metrics
    pub fn export_icmp_metrics(&self, ...) // ICMP metrics
}
```

### Metric Recording Flow

1. **Probe Execution** - HTTP/ICMP probe runs and collects timing data
2. **Metric Extraction** - `extract_metrics_values()` prepares values for export
3. **Async Export** - `tokio::spawn` sends metrics without blocking probes
4. **SDK Processing** - OpenTelemetry SDK batches and exports to OTLP endpoint
5. **Backend Reception** - OTLP collector receives and processes metrics

### Resource Attributes

All metrics include resource attributes from configuration:

```rust
Resource::new(vec![
    KeyValue::new("service.name", service_name),
    KeyValue::new("service.namespace", service_namespace),
    KeyValue::new("service.version", service_version),
    KeyValue::new("service.instance.id", service_instance_id),
])
```

## Usage Example

```rust
use exporter::otel::metrics::MetricsExporter;
use std::collections::HashMap;

// Create exporter with labels
let mut labels = HashMap::new();
labels.insert("target".to_string(), "https://example.com".to_string());
labels.insert("zone".to_string(), "eu-west-1".to_string());

let exporter = MetricsExporter::new(labels);

// Export HTTP metrics
exporter.export_metrics(
    1,      // up
    1,      // success
    25,     // dns_duration_ms
    200,    // status_code
    450,    // http_duration_ms
    Some(120), // tls_duration_ms
    Some(95),  // tls_handshake_ms
    Some(1735689600), // cert_expiration_ts
    Some(1704153600), // cert_begin_ts
);

// Export ICMP metrics
exporter.export_icmp_metrics(
    1,  // up
    15, // rtt_ms
);
```

## Testing

### Unit Tests

```bash
cargo test --package exporter --lib otel::tests
```

### Integration Testing

```bash
# Start Alloy
docker-compose -f dev/docker-compose-alloy-otel.yml up -d

# Run Zookoo
cargo run -- --config dev/conf/zookoo-alloy-otel-config.toml

# Verify metrics in Prometheus
curl 'http://localhost:9092/api/v1/query?query=probe_http_request_total'
```

### Metrics Validation

```bash
# Check all Zookoo metrics
curl -s 'http://localhost:9092/api/v1/label/__name__/values' | \
  jq '.data[] | select(startswith("probe_"))'

# Query specific metric with labels
curl -s 'http://localhost:9092/api/v1/query?query=probe_http_request_duration_milliseconds{target="https://example.com"}' | \
  jq '.data.result'
```

## Troubleshooting

### Common Issues

**1. Metrics not appearing in backend**

- Verify OTLP endpoint is reachable: `curl -v http://otel-collector:4317`
- Check Zookoo logs for connection errors
- Validate collector is receiving data: `docker logs otel-collector`
- Ensure `service_name` is configured in `[exporter.metrics]`

**2. TLS connection errors**

```
Error: failed to connect to endpoint
```

Solution: Set `tls_insecure = true` or provide valid `cert_path`:

```toml
[exporter.otel]
url = "https://otel-collector:4317"
cert_path = "/etc/ssl/certs/ca-bundle.crt"
tls_insecure = false
```

**3. Authentication failures**

```
Error: status: Unauthenticated
```

Solution: Add authentication credentials:

```toml
[exporter.otel.auth]
username = "your-username"
password = "your-password"
```

**4. High memory usage**

The SDK batches metrics before export. Adjust batch settings in collector:

```yaml
processors:
  batch:
    timeout: 5s
    send_batch_size: 512  # Reduce from default 1024
```

## Performance Considerations

### Async Export

Metrics are exported asynchronously to avoid blocking probe execution:

```rust
tokio::spawn(async move {
    exporter.export_metrics(...).await;
});
```

### Histogram Performance

Histograms use explicit buckets (not exponential) for predictable performance. Each histogram has ~10 buckets.

### Batch Processing

The OpenTelemetry SDK automatically batches metrics. Default settings:

- **Batch timeout**: 10 seconds
- **Batch size**: 1024 metrics
- **Queue size**: 2048 metrics

## Migration from Prometheus

### Metric Name Mapping

| Prometheus Metric | OTLP Metric | Notes |
|-------------------|-------------|-------|
| `http_probe_success` | `probe_http_request_total{status="success"}` | Counter instead of gauge |
| `http_probe_duration_seconds` | `probe_http_request_duration_milliseconds` | Milliseconds, histogram |
| `http_probe_dns_duration_seconds` | `probe_dns_lookup_duration_milliseconds` | Milliseconds, histogram |
| `icmp_probe_success` | `probe_icmp_request_total{status="success"}` | Counter instead of gauge |
| `icmp_probe_rtt_seconds` | `probe_icmp_rtt_milliseconds` | Milliseconds, histogram |

### Query Adjustments

**Prometheus:**
```promql
http_probe_duration_seconds{target="https://example.com"}
```

**OTLP (via Prometheus):**
```promql
histogram_quantile(0.95, 
  probe_http_request_duration_milliseconds_bucket{target="https://example.com"}
)
```

## References

- [OpenTelemetry Rust SDK](https://github.com/open-telemetry/opentelemetry-rust)
- [OTLP Specification](https://opentelemetry.io/docs/specs/otlp/)
- [Grafana Alloy OTLP](https://grafana.com/docs/alloy/latest/reference/components/otelcol.receiver.otlp/)
- [OpenTelemetry Collector](https://opentelemetry.io/docs/collector/)

---

**For more information, see the main [Zookoo documentation](../../../README.md).**
