---
sidebar_position: 3
---

# Exporters

## Open Telemetry

### Configuration

### Examples

```toml
[defaults]
log_level = "info"
self_monitoring = { enable = false }
probe_location = { latitude = 48.858370, longitude = 2.29448 }
probe_zone = "DEV"

[exporter.otel]
url = "http://localhost:4317"
tls_insecure = true

[http]
scrape_interval = "10s"
targets = [
    { url = "https://example.com", labels = { zone = "eu-west-1", env = "production" } },
    { url = "https://httpbin.org/status/200", labels = { zone = "us-east-1", env = "staging" } },
    { url = "https://www.google.com", labels = { zone = "eu-west-1", env = "production" } }
]

[icmp]
scrape_interval = "5s"
targets = [
    { target = "8.8.8.8", labels = { zone = "global", env = "production", name = "google-dns" } },
    { target = "1.1.1.1", labels = { zone = "global", env = "production", name = "cloudflare-dns" } }
]
```

## Prometheus Remote Write

Prometheus Remote Write exporter allows you to send metrics to any backend that supports Prometheus Remote Write protocol, such as Prometheus, Cortex, Thanos, Mimir, VictoriaMetrics, etc.

### Configuration

`url` (string, required): The URL of the Prometheus Remote Write endpoint.
`job` (string, optional): The job label to attach to all metrics sent to the remote write endpoint.
`instance` (string, optional): The instance label to attach to all metrics sent to the remote write endpoint.

### Examples

```toml
[defaults]
log_level = "info"
self_monitoring = { enable = false }
probe_location = { latitude = 48.858370, longitude = 2.29448 }
probe_zone = "DEV"

[exporter.prometheus_remote_write]
url = "http://localhost:9999/api/v1/metrics/write"
job = "zookoo-stm"
instance = "zookoo-dev-prometheus"

[http]
scrape_interval = "10s"
targets = [
    { url = "https://example.com", labels = { zone = "eu-west-1", env = "production" } },
    { url = "https://httpbin.org/status/200", labels = { zone = "us-east-1", env = "staging" } },
    { url = "https://www.google.com", labels = { zone = "eu-west-1", env = "production" } }
]

[icmp]
scrape_interval = "5s"
targets = [
    { target = "8.8.8.8", labels = { zone = "global", env = "production", name = "google-dns" } },
    { target = "1.1.1.1", labels = { zone = "global", env = "production", name = "cloudflare-dns" } }
]
```

## TimeScaleDB

### Configuration

`connection_string` (string, required): The connection string to connect to the TimeScaleDB database.

### Examples

```toml
[defaults]
log_level = "info"
self_monitoring = { enable = false }
probe_location = { latitude = 48.858370, longitude = 2.29448 }
probe_zone = "DEV"

[exporter.timescale]
connection_string = "postgresql://zookoo:zookoo@timescaledb:5432/zookoo"
# Optional: Specify database schema (default: "public")
schema = "monitoring"

[http]
targets = [
    { url = "https://example.com", labels = { zone = "eu-west-1", env = "production" } },
    { url = "https://httpbin.org/status/200", labels = { zone = "us-east-1", env = "staging" } },
    { url = "https://www.google.com", labels = { zone = "eu-west-1", env = "production" } }
]

[icmp]
targets = [
    { ipv4 = "8.8.8.8", labels = { zone = "global", env = "production", name = "google-dns" } },
    { ipv4 = "1.1.1.1", labels = { zone = "global", env = "production", name = "cloudflare-dns" } }
]
```