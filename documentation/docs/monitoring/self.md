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

It is enabled by default when self-monitoring is turned on and send profiles to the configured Pyroscope endpoint.

_Used mainly in development and debugging scenarios._

## Logging

Zookoo uses structured logging to provide detailed insights into its operations. Logs include key-value pairs that make it easy to filter and analyze log data.

It is enabled by default when self-monitoring is turned on and send logs to the configured OpenTelemetry endpoint.

_Used mainly in development and debugging scenarios._

## Tracing

Zookoo comes with built-in trace monitoring capabilities.

Each scrape operation is traced, allowing you to follow the complete lifecycle of a probe from initiation to completion.

It is enabled by default when self-monitoring is turned on and send traces to the configured OpenTelemetry endpoint.

_Used mainly in development and debugging scenarios._