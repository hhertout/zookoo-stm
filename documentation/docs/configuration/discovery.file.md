# discovery.file

## Arguments

You can use the following arguments with discovery.file:

| Name | Type | Description | Default | Required |
|------|------|-------------|---------|----------|
| path | list(string) | List of file paths to discover targets from. Supports JSON format. | | yes |
| labels | map(string) | Static labels to attach to all discovered targets. | | no |
| scrape_interval | duration | Interval at which targets should be scraped. | 1m | no |
| probe_type | string | Type of probe (http or icmp) for discovered targets. | | no |

## Example

```hcl
discovery "file" "json_targets" {
  path = ["./targets.json"]
}
```