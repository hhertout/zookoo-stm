defaults {
  log_level = "info"
  probe_zone = "FRA"
  
  probe_location {
    latitude = 48.858370
    longitude = 2.29448
  }
  
  self_monitoring {
    enable = true
    otel_endpoint = "https://otel-grpc.neryolab.com"
    pyroscope_endpoint = "https://otel-pyroscope.neryolab.com"
    service_name = "zookoo"
    env = "test"
  }
}

probe "http" "api_monitoring" {
  targets = [
    {
      url = "https://example.com"
      labels = {
        zone = "eu-west-1"
        env = "production"
      }
    },
    {
      url = "https://httpbin.org/status/200"
      labels = {
        zone = "us-east-1"
        env = "staging"
      }
    },
    {
      url = "https://www.google.com"
      labels = {
        zone = "eu-west-1"
        env = "production"
      }
    }
  ]

  forward_to = [exporter.otlp.main]
}

probe "http" "json_monitoring" {
  target_from = discovery.file.json_targets
  forward_to = [exporter.otlp.main]
}

exporter "otlp" "main" {
  url = "https://otel-grpc.neryolab.com"
  tls_insecure = false
}

discovery "file" "json_targets" {
  path = ["/etc/zookoo/targets.json"]
}
