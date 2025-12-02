# Zookoo configuration for Prometheus Remote Write exporter
# Use with: docker-compose -f dev/docker-compose-prometheus.yml up

defaults {
  log_level = "info"
  
  self_monitoring {
    enable = false
  }
  
  probe_location {
    latitude = 48.858370
    longitude = 2.29448
  }
  
  probe_zone = "DEV"
}

exporter "prometheus_remote_write" "main" {
    url = "http://localhost:9999/api/v1/metrics/write"
    job = "zookoo-stm"
    instance = "zookoo-dev-prometheus"
}

probe "http" "api_monitoring" {
  scrape_interval = "10s"
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
}

probe "icmp" "infrastructure_ping" {
  scrape_interval = "5s"
    targets = [
      {
        target = "8.8.8.8"
        labels = {
          zone = "global"
          env = "production"
          name = "google-dns"
        }
      },
      {
        target = "1.1.1.1"
        labels = {
          zone = "global"
          env = "production"
          name = "cloudflare-dns"
        }
      }
    ]
}
