# Zookoo Configuration - TimescaleDB Exporter

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

exporter "timescale" "main" {
  connection_string = "postgresql://zookoo:zookoo@timescaledb:5432/zookoo"
    # Optional: Specify database schema (default: "public")
    schema = "monitoring"
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
}

probe "icmp" "infrastructure_ping" {
    targets = [
      {
        ipv4 = "8.8.8.8"
        labels = {
          zone = "global"
          env = "production"
          name = "google-dns"
        }
      },
      {
        ipv4 = "1.1.1.1"
        labels = {
          zone = "global"
          env = "production"
          name = "cloudflare-dns"
        }
      }
    ]
}
