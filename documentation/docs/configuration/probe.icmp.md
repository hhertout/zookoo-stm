# probe.icmp

ICMP probing allows you to monitor the availability and responsiveness of networked devices by sending ICMP Echo Request packets (pings) and measuring the response times.

## Arguments

You can use the following arguments with probe.icmp:

| Name | Type | Description | Default | Required |
|------|------|-------------|---------|----------|
| scrape_interval | duration | Interval at which targets should be pinged. | 1m | no |
| targets | list(object) | List of ICMP targets to probe. See target arguments below. | | no |
| target_from | reference | Reference to a discovery configuration to load targets from. | | no |
| forward_to | list(reference) | List of exporter references to send metrics to. | | yes |

### Target Arguments

| Name | Type | Description | Default | Required |
|------|------|-------------|---------|----------|
| ipv4 | string | IPv4 address to ping. | | no* |
| fqdn | string | Fully qualified domain name to ping (will be resolved to IP). | | no* |
| timeout_sec | integer | Timeout in seconds for the ICMP request. | 15 | no |
| labels | map(string) | Labels to attach to this target. | | no |

*At least one of `ipv4` or `fqdn` must be provided.

## Example

### Plain text configuration with IPv4 addresses

```hcl
probe "icmp" "servers" {
  scrape_interval = "30s"
  targets = [
    {
      ipv4 = "8.8.8.8"
      timeout_sec = 10
      labels = {
        service = "dns_google"
        env = "production"
      }
    },
    {
      ipv4 = "1.1.1.1"
      timeout_sec = 10
      labels = {
        service = "dns_cloudflare"
        env = "production"
      }
    }
  ]

  forward_to = [exporter.otlp.otlp]
}
```

### Configuration with FQDN targets

```hcl
probe "icmp" "endpoints" {
  scrape_interval = "60s"
  targets = [
    {
      fqdn = "google.com"
      timeout_sec = 15
      labels = {
        service = "google"
        env = "production"
      }
    },
    {
      fqdn = "cloudflare.com"
      timeout_sec = 15
      labels = {
        service = "cloudflare"
        env = "production"
      }
    }
  ]

  forward_to = [exporter.otlp.otlp]
}
```

### From a discovered file

```hcl
discovery "file" "icmp_targets" {
  path = ["./targets/icmp.json"]
}

probe "icmp" "discovered" {
  target_from = discovery.file.icmp_targets
  forward_to = [exporter.otlp.otlp]
}
```

## Resources

None yet.