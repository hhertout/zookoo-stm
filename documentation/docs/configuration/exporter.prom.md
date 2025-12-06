# exporter.prometheus_remote_write

Prometheus Remote Write exporter allows you to send monitoring data to compatible backends that support the Prometheus remote_write API. This enables integration with various time-series databases and monitoring systems.

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

## Limitations

:::warning
This exporter only supports **gauge metrics**. Histograms with buckets (`_bucket`, `_sum`, `_count` suffixes) are **not supported**.

If you need histogram metrics for percentile calculations or latency distribution analysis, use the **OTLP exporter** instead, which provides full histogram support.
:::


## Arguments

You can use the following arguments with exporter.prometheus_remote_write:

| Name | Type | Description | Default | Required |
|------|------|-------------|---------|----------|
| url | string | Remote write URL endpoint (e.g., http://localhost:9999/api/v1/metrics/write). | | yes |
| job | string | Prometheus job label to attach to all metrics. | zookoo-stm | no |
| instance | string | Prometheus instance label to attach to all metrics. | | no |
| auth | object | Authentication configuration (see auth block below). | | no |

### Auth Block

| Name | Type | Description | Default | Required |
|------|------|-------------|---------|----------|
| username | string | Username for basic authentication. | | no |
| password | string | Password for basic authentication. | | no |
| bearer | string | Bearer token for authentication. | | no |

## Example

```hcl
exporter "prometheus_remote_write" "main" {
    url = "http://localhost:9999/api/v1/metrics/write"
}
```

## Resources

- [Prometheus Remote Write Spec](https://prometheus.io/docs/prometheus/latest/querying/api/#remote-write-receiver)
- [Grafana Alloy Documentation](https://grafana.com/docs/alloy/latest/)
- [prometheus.receive_http Component](https://grafana.com/docs/alloy/latest/reference/components/prometheus/prometheus.receive_http/)
- [Grafana Mimir](https://grafana.com/oss/mimir/)