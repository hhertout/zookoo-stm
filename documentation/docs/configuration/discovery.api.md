# discovery.api

Discover targets from an HTTP API endpoint returning JSON.

This discovery mechanism allows you to dynamically fetch a list of targets from a specified HTTP endpoint. The endpoint should return a JSON array of target objects that conform to the expected target structure for the probe type being used.

The list is refreshed at regular intervals defined by the `refresh_interval` parameter, allowing dynamic updates of the target list without requiring a restart of the monitoring system.

:::warning
If the URL endpoint is unreachable or returns invalid data at some point, the previously cached targets will continue to be used until a successful fetch occurs.

If the URL endpoint is unreachable at startup, the discovery will fail to initialize.
:::

:::note
If the endpoint returns an empty array, it is treated as having no targets rather than an error.
:::

## Arguments

You can use the following arguments with discovery.api:

| Name             | Type        | Description                                                                            | Default | Required |
| ---------------- | ----------- | -------------------------------------------------------------------------------------- | ------- | -------- |
| url              | string      | HTTP endpoint returning the discovered targets as JSON (an array).                     |         | yes      |
| headers          | map(string) | Extra HTTP headers added to the request.                                               | empty   | no       |
| basic_auth       | string      | Value inserted into the `Authorization` header (used as-is).                           |         | no       |
| bearer           | string      | Bearer token for authentication (sets the `Authorization` header to `Bearer <token>`). |         | no       |
| refresh_interval | duration    | How often the API is polled to refresh the cached targets.                             | 1h      | no       |

`refresh_interval` accepts the same values as `RefreshInterval` in the configuration crate (e.g. `"5s"`, `"30s"`, `"1m"`, `"1h"`, `"1d"`, ...).

### API Response Format

To use this discovery mechanism, the specified HTTP endpoint must return a JSON array of target objects. Each target object should conform to the expected structure for the probe type being used. For example, if you are using this discovery with an HTTP probe, each target object should include fields like `url`, `method`, etc.

This format is **based on the target structure defined in the [respective HTTP probe documentation](/docs/configuration/probe.http#target-arguments) or [respective ICMP probe documentation](/docs/configuration/probe.icmp#target-arguments)** depending on the probe type. Ensure that the JSON response from your API matches the expected format for seamless integration.

**Example JSON Response:**

```json
[
  {
    "url": "https://example.com/health",
    "method": "GET",
    "expected_status_code": 200,
    "timeout_sec": 10,
    "labels": {
      "service": "example_service",
      "env": "production"
    }
  },
  {
    "url": "https://another-service.com/status",
    "method": "GET",
    "expected_status_code": 200,
    "timeout_sec": 15,
    "labels": {
      "service": "another_service",
      "env": "staging"
    }
  }
]
```

## Example

```hcl
discovery "api" "dynamic_targets" {

  url = "https://example.com/api/targets"

  headers = {
    "X-SPECIFIC-HEADER" = "my_header_value"
  }

  basic_auth = "ZHVtbXk6ZHVtbXk=" # base64(user:pass)

  refresh_interval = "1h"
}

probe "http" "dynamic_http_checks" {
  scrape_interval = "30s"
  target_from = discovery.api.dynamic_targets
  refresh_interval = "15s"

  forward_to = [exporter.otlp.default]
}
```
