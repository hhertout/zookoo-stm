# probe.http

HTTP probing allows you to monitor the availability and responsiveness of web services by sending HTTP requests and evaluating the responses.

## Arguments

You can use the following arguments with probe.http:

| Name | Type | Description | Default | Required |
|------|------|-------------|---------|----------|
| scrape_interval | duration | Interval at which targets should be probed. | 1m | no |
| targets | list(object) | List of HTTP targets to probe. See target arguments below. | | no |
| target_from | reference | Reference to a discovery configuration to load targets from. | | no |
| forward_to | list(reference) | List of exporter references to send metrics to. | | yes |

### Target Arguments

| Name | Type | Description | Default | Required |
|------|------|-------------|---------|----------|
| url | string | HTTP URL to probe. | | yes |
| method | string | HTTP method to use (GET, POST, etc.). | GET | no |
| expected_status_code | integer | Expected HTTP status code for successful probe. | 200 | no |
| timeout_sec | integer | Timeout in seconds for the HTTP request. | 15 | no |
| headers | map(string) | Custom HTTP headers to send with the request. | | no |
| labels | map(string) | Labels to attach to this target. | | no |
| auth | object | Authentication configuration (see auth block below). | | no |
| follow_redirect | boolean | Whether to follow HTTP redirects. | false | no |
| skip_tls | boolean | Skip TLS certificate validation. | false | no |

### Auth Block

| Name | Type | Description | Default | Required |
|------|------|-------------|---------|----------|
| username | string | Username for basic authentication. | | no |
| password | string | Password for basic authentication. | | no |
| bearer | string | Bearer token for authentication. | | no |

## Example

### Plain text configuration

```hcl
probe "http" "google_check" {
  scrape_interval = "30s"
  targets = [
    {
      url = "https://www.google.com"
      method = "GET"
      expected_status_code = 200
      labels = {
        service = "google"
        env = "test"
      }
    }
  ]

  forward_to = [exporter.otlp.otlp]
}
```

### From a JSON file

```hcl
discovery "file" "json_targets" {
  path = ["./targets.json"]
}

probe "http" "test" {
  target_from = discovery.file.json_targets
  forward_to = [exporter.otlp.otlp]
}
```

## Resources