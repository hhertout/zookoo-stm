---
sidebar_position: 1
---

# Presentation

Welcome to **ZookooZookoo**, a synthetic monitoring solution designed to help you monitor your applications and services effectively.

ZookooZookoo is built with Rust, leveraging its performance and reliability to provide a robust monitoring solution. It is fully compliant with Open Telemetry standards, allowing you to collect and export metrics seamlessly.

ZookooZookoo is designed to be user-friendly and easy to set up, making it accessible for developers and operations engineers alike. With a simple configuration file, you can quickly define your monitoring targets and start collecting metrics.

This project provides an alternative to Blackbox, offering improved performance and greater configurability.

# Why this project?

One of the biggest pain points with the Prometheus Blackbox Exporter is its fragmented and rigid configuration model. You need to define:

- Modules in a separate blackbox.yml file

- Targets through Prometheus relabeling or external files

- Scraping rules in prometheus.yml

And sometimes even dynamic reloading via third-party tools
This makes deployment cumbersome, error-prone, and difficult to maintain at scale. And not user friendly at for beginners.

### With ZookooZookoo, it’s different:

We use a single, unified config file written in clean and human-friendly TOML:

```toml
[exporter.otel]
url = "http://localhost:4317"

[exporter.kafka]
broker = "localhost:9092"
topic = "metrics"

[http]
targets = [
  { url = "https://google.com", labels = { env = "prod", service = "search" }, scrap_interval = "10s" },
  { url = "https://chatgpt.com", labels = { env = "test", service = "ai" }, scrap_interval = "5s" },
]
```

- Define multiple exporters (OpenTelemetry, Kafka, InfluxDB...)
- Assign custom labels and individual scrape intervals per target
- Manage everything from one file — no Prometheus required

Optional: you can even load targets from a JSON file or API, allowing full dynamic configuration.

### Simpler to deploy, easier to scale

Whether you run it as a standalone binary or integrated into your observability pipeline, our tool eliminates the YAML juggling and makes configuration declarative, centralized, and easy to automate.

No more deciphering Prometheus relabeling rules just to monitor a simple endpoint.

### Advantages

This tool was created to have a simpler, more flexible, and more powerful alternative to the Prometheus Blackbox Exporter. It aims to provide a better user experience by:

- Simpler way to put in place in your environment
- Offering a more predictable performance profile
- Being easy to configure and integrate in modern observability stacks
- Allowing you to scrape targets and send data to your chosen service — all powered by Rust’s blazingly fast performance.

# Features

- **Scraping targets**:

  - HTTP
  - HTTPS

- **Exporters**:
  - OpenTelemetry

## Getting Started

Let's discover **ZookooZookoo in less than 5 minutes**.

Get started by **creating a new scraping engine**.

### What you'll need

- [Docker](https://www.docker.com/)

## Going further

Coming soon.
