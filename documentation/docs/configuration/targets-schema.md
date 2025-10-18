---
sidebar_position: 2
---

# Targets Schema

## Block

- **`url`**:

The URL of the target to scrape.

- **`labels `**:

A map of labels to attach to the target. Useful for filtering and grouping metrics.

- **`scrap_interval`**:

The interval at which to scrape the target. Default is `"1m"`.

- **`method`**:

The HTTP method to use for scraping. Default is `"GET"`.

- **`headers`**:

A map of headers to include in the request. Useful for authentication or custom headers.

- **`expected_status_code`**:

The expected HTTP status code for a successful response. Default is `200`.

## Example

```toml
[http]
targets = [
    { url = "https://example.com/", labels = { "env" = "production", "service_name" = "web" }, scrap_interval = "30s", method = "GET"}
]
```
