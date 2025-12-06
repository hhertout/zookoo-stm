---
sidebar_position: 1
---

# Enable Internal Monitoring

To monitor the health and performance of Zookoo, you can enable internal monitoring features. profiling, and tracing.

All the data is exported by using open telemetry and pyroscope.

```hcl
default {
    self_monitoring = { 
        enable = true, # default is false
        otel_endpoint = "http://localhost:4317", 
        pyroscope_endpoint = "http://localhost:9999" 
    } 
}
```

## Pyroscope

Zookoo integrates with Pyroscope to provide continuous profiling of the application. This helps in identifying performance bottlenecks and understanding resource usage.

_Used mainly in development and debugging scenarios._

## Tracing

Zookoo comes with built-in trace monitoring capabilities.

Each scrape operation is traced, allowing you to follow the complete lifecycle of a probe from initiation to completion.

_Used mainly in development and debugging scenarios._