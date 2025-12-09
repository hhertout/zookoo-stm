# defaults

The `defaults` block allows you to configure global default values that apply to all probes and exporters. These settings control logging, service identification, probe location metadata, and self-monitoring capabilities.

## Arguments

| Name | Type | Description | Default | Required |
|------|------|-------------|---------|----------|
| log_level | string | Log level for the application (debug, info, warn, error). | info | no |
| job | string | Default job label attached to all exported metrics. | zookoo | no |
| service_name | string | Service name used for identification in metrics and traces. | zookoo | no |
| probe_zone | string | Zone identifier for the probe (e.g., eu-west-1, us-east-1). | | no |
| probe_location | object | Geographic location of the probe (see probe_location block below). | | no |
| self_monitoring | object | Self-monitoring configuration (see self_monitoring block below). | | no |
| metric_prefix | string | Default prefix for all metric names across all exporters. | probe_ | no |

## probe_location Block

The `probe_location` block allows you to specify the geographic coordinates of the probe. This is useful for geo-distributed monitoring setups.

| Name | Type | Description | Default | Required |
|------|------|-------------|---------|----------|
| latitude | float | Latitude coordinate of the probe location. | | yes |
| longitude | float | Longitude coordinate of the probe location. | | yes |

## self_monitoring Block

The `self_monitoring` block enables internal observability for Zookoo itself. This is useful for debugging and monitoring the health of the probe.

| Name | Type | Description | Default | Required |
|------|------|-------------|---------|----------|
| enable | boolean | Enable self-monitoring. | false | no |
| otel_endpoint | string | OpenTelemetry collector endpoint for traces and metrics. | http://localhost:4317 | no |
| pyroscope_endpoint | string | Pyroscope endpoint for continuous profiling. | http://localhost:9999 | no |
| service_name | string | Service name for self-monitoring data. | zookoo | no |
| env | string | Environment label for self-monitoring data. | development | no |
| tls_ignore | boolean | Skip TLS certificate verification for self-monitoring endpoints. | false | no |

## Example

### Basic Configuration

```hcl
defaults {
  log_level = "info"
  job = "zookoo"
  service_name = "zookoo"
}
```

### With Probe Location and Metric Prefix

```hcl
defaults {
  log_level = "info"
  job = "zookoo-eu"
  service_name = "zookoo"
  probe_zone = "eu-west-1"
  metric_prefix = "zookoo_"

  probe_location {
    latitude = 48.8566
    longitude = 2.3522
  }
}
```

`metric_prefix` is an optional string that, if provided, will be prepended to all metric names exported by all exporters. This can be useful for namespacing metrics in environments where multiple applications or services are sending metrics to the same backend. If not specified, the default prefix is `probe_`.

### With Self-Monitoring

```hcl
defaults {
  log_level = "debug"
  job = "zookoo"
  service_name = "zookoo"

  self_monitoring {
    enable = true
    otel_endpoint = "http://otel-collector:4317"
    pyroscope_endpoint = "http://pyroscope:4040"
    service_name = "zookoo-probe"
    env = "production"
    tls_ignore = false
  }
}
```

### Full Configuration

```hcl
defaults {
  log_level = "info"
  job = "zookoo-prod"
  service_name = "zookoo"
  probe_zone = "us-east-1"
  metric_prefix = "probe_"

  probe_location {
    latitude = 40.7128
    longitude = -74.0060
  }

  self_monitoring {
    enable = true
    otel_endpoint = "http://otel-collector:4317"
    pyroscope_endpoint = "http://pyroscope:4040"
    service_name = "zookoo"
    env = "production"
    tls_ignore = false
  }
}
```

## Resources

- [OpenTelemetry Semantic Conventions](https://opentelemetry.io/docs/specs/semconv/)
- [Pyroscope Documentation](https://pyroscope.io/docs/)
