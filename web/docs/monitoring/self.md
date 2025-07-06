---
sidebar_position: 1
---

# Enable Internal Monitoring

To monitor the health and performance of ZookooZookoo, you can enable internal monitoring features. profiling, and tracing.

All the data is exported by using open telemetry and pyroscope.

```toml
[default]
self_monitoring = { enable = true, otel_endpoint = "http://localhost:4317", pyroscope_endpoint = "http://localhost:9999" } # default is false
```

## Pyroscope

## Tracing
