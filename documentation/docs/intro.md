---
sidebar_position: 1
---

# Presentation

Welcome to **Zookoo**, a synthetic monitoring solution designed to help you monitor your applications and services effectively.

Zookoo is built with Rust, leveraging its performance and reliability to provide a robust monitoring solution. It is fully compliant with Open Telemetry standards, allowing you to collect and export metrics seamlessly.

Zookoo is designed to be user-friendly and easy to set up, making it accessible for developers and operations engineers alike. With a simple configuration file, you can quickly define your monitoring targets and start collecting metrics.

This project provides an alternative to Blackbox, offering improved performance and greater configurability.

### Easy configuration

Define your monitoring setup by using a clear and concise HCL configuration file. 
Configure the pipeline you need in a single place. It offer flexibility and simplicity.

Use a single, unified config file written in HCL:

```hcl
defaults {
  log_level = "info"
  probe_zone = "eu-west-1"
  service_name = "zookoo"
  job = "zookoo"

  probe_location {
    latitude = 48.858370
    longitude = 2.29448
  }
}

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

  forward_to = [exporter.otel.otlp]
}

exporter "otel" "otlp" {
  url = "http://localhost:4317"
  tls_insecure = true
}

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
  - Prometheus Remote Write
  - TimescaleDB (PostgreSQL with time-series extension)

## Getting Started

Let's discover **Zookoo in less than 5 minutes**.

Get started by **creating a new scraping engine**.

### What you'll need

- [Docker](https://www.docker.com/)

## Going further

Coming soon.
