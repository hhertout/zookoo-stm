---
sidebar_position: 2
---

# ICMP metrics

This page lists all metrics exported by the ICMP probe. The ICMP probe sends ping requests to the configured targets and reports reachability and round-trip time.

## Labels

Every ICMP metric is exported with the following labels.

| Label | Description |
|---|---|
| `instance` | The target hostname or IPv4 address being pinged (e.g. `8.8.8.8`) |
| `probe` | Always `icmp` |
| custom labels | Any labels defined in the target configuration (e.g. `service`, `env`) |

## Metrics

| Metric | Type | Unit | Description |
|---|---|---|---|
| `up` | Gauge | — | `1` if the ping succeeded, `0` if the target did not respond |
| `rtt_ms` | Gauge | ms | Round-trip time of the ping request |

`up` is set to `0` when the target is unreachable or when the ICMP reply does not arrive within the timeout. `rtt_ms` is only meaningful when `up` is `1`.

## Example output

The following example shows a typical set of metrics exported after a successful ICMP probe.

```
up{instance="8.8.8.8", probe="icmp", service="dns", env="prod"} 1
rtt_ms{instance="8.8.8.8", probe="icmp", service="dns", env="prod"} 3
```
