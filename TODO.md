# TODO List

- Global:
  - check histogram metrics -> doesn't work properly (to confirm)
  - Unit Tests...
  - Configuration Hot Module Reload
  - UI
  - Load testing (k6 ?) & benchmarking (vs blackbox-exporter ?)
  - Clusterting - Ring to handle multi containerization / k8s with the same config target

- Probers:

  - TCP connect probe
  - UDP probe
  - DNS probe

- Defaults:

  - Headers
  - Timeout

- Targets:
  - Probe HTTP redirect & follow redirect
  - Expected content
  - Expected content type
  - Proxy configuration
  - GRPC request handler
  - DNS test and expiration date of a dns name
  - Skip tls option on requests

- Discovery:
  - Discovery API http targets (important)

- Exporter:
  - Kafka exporter
  - InfluxDB exporter
  - MongoDB exporter
  - TimescaleDB / PostgreSQL exporter
  - Referential building
