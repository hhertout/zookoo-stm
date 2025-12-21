# probe.tcp

TCP probing checks whether a TCP port on a host is reachable by attempting a TCP connect and recording connect success and timing.

## Arguments

You can use the following arguments with `probe.tcp`:

| Name            | Type            | Description                                                  | Default | Required |
| --------------- | --------------- | ------------------------------------------------------------ | ------- | -------- |
| scrape_interval | duration        | Interval at which targets should be probed.                  | 1m      | no       |
| targets         | list(object)    | List of TCP targets to probe. See target arguments below.    |         | no       |
| target_from     | reference       | Reference to a discovery configuration to load targets from. |         | no       |
| forward_to      | list(reference) | List of exporter references to send metrics to.              |         | yes      |

### Target Arguments

The TCP target object expects the following fields:

| Name        | Type        | Description                                                                                                               | Default | Required |
| ----------- | ----------- | ------------------------------------------------------------------------------------------------------------------------- | ------- | -------- |
| target      | string      | A target string in the form `host:port`, `ip:port` or `fqdn:port`. The probe will attempt a TCP connect to this endpoint. |         | yes      |
| timeout_sec | integer     | Timeout in seconds for the TCP connect attempt.                                                                           | 15      | no       |
| labels      | map(string) | Labels to attach to this target.                                                                                          |         | no       |

Notes:

- The `target` value must include a port. Targets without a port will fail the connect attempt (tests cover this edge case).
- You can use hostnames (e.g. `localhost:1234`) or FQDNs — they will be resolved before connecting. Use `.invalid` domains in tests for guaranteed DNS failures.

## Example

### Plain text configuration

```hcl
probe "tcp" "service_tcp" {
	scrape_interval = "30s"
	targets = [
		{
			target = "10.0.0.5:80"
			timeout_sec = 5
			labels = {
				service = "backend"
				env = "staging"
			}
		}
	]

	forward_to = [exporter.otlp.otlp]
}
```

### Hostname (FQDN) example

```hcl
probe "tcp" "local_check" {
	targets = [
		{ target = "localhost:8080" }
	]
	forward_to = [exporter.prometheus.prom]
}
```

### From discovery (file)

```hcl
discovery "file" "tcp_targets" {
	path = ["./targets/tcp.json"]
}

probe "tcp" "discovered" {
	target_from = discovery.file.tcp_targets
	forward_to = [exporter.otlp.otlp]
}
```

## Resources

None yet.
