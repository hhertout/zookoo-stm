# Zookoo configuration - using discovery.file for fair comparison with Blackbox
# Uses prometheus_remote_write directly to Prometheus (like Blackbox+OTEL does)

defaults {
  log_level = "info"
  probe_zone = "benchmark"
  job = "zookoo-benchmark"
}

exporter "prometheus_remote_write" "prom" {
  url = "http://prometheus:9090/api/v1/write"
  job = "zookoo-benchmark"
  instance = "zookoo"
}

discovery "file" "targets" {
  path = ["/config/targets.json"]
}

probe "http" "benchmark" {
  scrape_interval = "5s"
  target_from = discovery.file.targets
  forward_to = [exporter.prometheus_remote_write.prom]
}
