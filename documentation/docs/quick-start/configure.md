---
sidebar_position: 1
---

# Create the configuration file

For configuration of Zookoo, you need to create a configuration file where you will describe your targets, and where you want to send the data generated from the scrapping.

This is a simple configuration file to scrape a single target.

```toml
# config.toml

[defaults]
log_level = "info"

[exporter.otel]
url = "http://localhost:4317"

[http]
targets = [
    { url = "https://google.com/", labels = { "env" = "test", "service_name" = "google" }, scrap_interval = "10s" },
]
```
