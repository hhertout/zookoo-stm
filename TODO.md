# TODO List

- Global:

  - Fix prod dockerfile `exec /usr/local/bin/zookoo: no such file or directory`
  - Check config function (main.rs)
  - check histogram metrics -> doesn't work properly (to confirm)
  - Unit Tests...
  - Configuration Hot Module Reload
  - Process Operator to add / restart the process via api
  - UI
  - Load testing (k6 ?) & benchmarking
  - Clusterting - Ring to handle multi containerization / k8s with the same config target

- Defaults:

  - Headers
  - Timeout

- Targets:

  - Labels on ICMP targets
  - Probe HTTP redirect & follow redirect
  - Expected content
  - Expected content type
  - Proxy configuration
  - GRPC request handler
  - DNS test and expiration date of a dns name
  - Skip tls option on requests

- Discovery:

  - Discovery API http targets

- Exporter:

  - Kafka exporter
  - InfluxDB exporter
  - MongoDB exporter
  - TimescaleDB / PostgreSQL exporter
  - Referential building
