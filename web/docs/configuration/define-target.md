---
sidebar_position: 1
---

# Define Target

## From the config file

```toml
[http]
targets = [
    { url = "https://google.com/", labels = { "env" = "test", "service_name" = "test" }, scrap_interval = "10s" },
    { url = "https://chatgpt.com/", labels = { "env" = "chat_gpt", "service_name" = "d4x" }, scrap_interval = "5s" },
]
```

## From a JSON file

```toml
[http]
targets_files = ["targets_zone1.json", "targets_zone2.json"]
```

## Target configuration

### Block

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
