---
sidebar_position: 1
---

# HTTP metrics

This page lists all metrics exported by the HTTP probe. These metrics are available for every scrape cycle and can be forwarded to any configured exporter.

## Labels

Every HTTP metric is exported with the following labels.

| Label | Description |
|---|---|
| `instance` | The target URL being probed (e.g. `https://www.google.com`) |
| `probe` | Always `http` |
| custom labels | Any labels defined in the target configuration (e.g. `service`, `env`) |

## Availability metrics

These metrics indicate whether the probe reached the target and whether the response matched the expected outcome.

| Metric | Type | Description |
|---|---|---|
| `up` | Gauge | `1` if the TCP connection to the target succeeded, `0` otherwise |
| `success` | Gauge | `1` if the HTTP status code matches `expected_status_code`, `0` otherwise |
| `status_code` | Gauge | The HTTP response status code returned by the target (e.g. `200`, `404`, `500`) |

## Timing metrics

These metrics break down the end-to-end request duration into individual phases.

| Metric | Type | Unit | Description |
|---|---|---|---|
| `dns_duration_ms` | Gauge | ms | Time taken to resolve the target hostname via DNS |
| `tcp_connect_duration_ms` | Gauge | ms | Time taken to establish the TCP connection |
| `time_to_first_byte_ms` | Gauge | ms | Time from sending the request to receiving the first byte of the response (TTFB) |
| `content_transfer_duration_ms` | Gauge | ms | Time to fully download the response body |
| `total_duration_ms` | Gauge | ms | Total end-to-end duration of the probe |
| `http_duration_ms` | Gauge | ms | Alias of `total_duration_ms`, retained for backwards compatibility |

## TLS metrics

These metrics are only exported when the target uses HTTPS and the TLS handshake completes successfully.

| Metric | Type | Unit | Description |
|---|---|---|---|
| `tls_handshake_ms` | Gauge | ms | Time taken to complete the TLS handshake |
| `cert_expiration_ts` | Gauge | Unix timestamp (s) | Certificate expiry date as a Unix epoch timestamp |
| `cert_begin_ts` | Gauge | Unix timestamp (s) | Certificate start-of-validity date as a Unix epoch timestamp |

Use `cert_expiration_ts` to build Grafana alerts that notify you before a certificate expires.

## Example output

The following example shows a typical set of metrics exported after a successful HTTPS probe.

```
up{instance="https://www.google.com", probe="http", service="google", env="prod"} 1
success{instance="https://www.google.com", probe="http", service="google", env="prod"} 1
status_code{instance="https://www.google.com", probe="http", service="google", env="prod"} 200
dns_duration_ms{instance="https://www.google.com", probe="http", service="google", env="prod"} 12
tcp_connect_duration_ms{instance="https://www.google.com", probe="http", service="google", env="prod"} 8
tls_handshake_ms{instance="https://www.google.com", probe="http", service="google", env="prod"} 35
time_to_first_byte_ms{instance="https://www.google.com", probe="http", service="google", env="prod"} 78
content_transfer_duration_ms{instance="https://www.google.com", probe="http", service="google", env="prod"} 4
total_duration_ms{instance="https://www.google.com", probe="http", service="google", env="prod"} 137
cert_expiration_ts{instance="https://www.google.com", probe="http", service="google", env="prod"} 1767225600
cert_begin_ts{instance="https://www.google.com", probe="http", service="google", env="prod"} 1759363200
```
