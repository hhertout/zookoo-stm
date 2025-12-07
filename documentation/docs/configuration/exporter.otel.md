# exporter.otel

OpenTelemetry (OTel) exporter allows you to send monitoring data to any backend that supports the OpenTelemetry Protocol (OTLP). This enables integration with a wide range of observability platforms and tools.

## Arguments

You can use the following arguments with exporter.otel:

| Name | Type | Description | Default | Required |
|------|------|-------------|---------|----------|
| url | string | OpenTelemetry collector endpoint URL (e.g., http://localhost:4317). | | yes |
| tls_insecure | boolean | Skip TLS certificate verification. | false | no |
| auth | object | Authentication configuration (see auth block below). | | no |
| cert_path | string | Path to a custom CA certificate file for TLS verification. | | no |
| metric_prefix | string | Optional prefix to prepend to all metric names. | probe_ | no |

`metric_prefix` is an optional string that, if provided, will be prepended to all metric names exported by this exporter. This can be useful for namespacing metrics in environments where multiple applications or services are sending metrics to the same backend. If not specified, prefix applied is `probe_`.

### Auth Block

| Name | Type | Description | Default | Required |
|------|------|-------------|---------|----------|
| username | string | Username for basic authentication. | | no |
| password | string | Password for basic authentication. | | no |
| bearer | string | Bearer token for authentication. | | no |

## Example

```hcl
exporter "otel" "default" {
  url = "http://localhost:4317"
  tls_insecure = true
}
```

## Resources

- [OpenTelemetry Protocol (OTLP) Exporter](https://opentelemetry.io/docs/instrumentation/exporters/otlp/)
- [OpenTelemetry Specification](https://opentelemetry.io/specs/)
