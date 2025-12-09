<p align="center">
  <img src=".github/assets/zookoo.png" width="250">

  <h1 align="center">Zookoo (on going)</h1>
    <p align="center">STM tool for scraping your targets and export your metrics !</p>
</p>

<p align="center">
    <img src="https://img.shields.io/badge/version-0.0.1-blue" alt="version">
    <a href="https://github.com/hhertout/rac_tool/actions">
      <img alt="Tests Passing" src="https://github.com/hhertout/zookoo-stm/actions/workflows/docker.yml/badge.svg" />
    </a>
    <a href="https://github.com/hhertout/rac_tool/actions">
      <img alt="Tests Passing" src="https://github.com/hhertout/zookoo-stm/actions/workflows/nightly.yml/badge.svg" />
    </a>
</p>

# Presentation

This project provides an alternative to Blackbox, offering improved performance and greater configurability.

It is built as a synthetic monitoring tool (STM) that allows you to scrape targets and send data to your chosen service — all powered by Rust’s blazingly fast performance.

**Scraping target** :

- HTTP
- HTTPS
- ICMP

**Exporter** :

- Open telemetry (http/protobuf & grpc)
- Prometheus Remote Write (compatible with Grafana Alloy, Prometheus, Mimir, etc.)
- TimescaleDB (PostgreSQL with time-series extension)

# Why this project?

In large-scale environments, we've found Blackbox Exporter to present limitations in terms of performance and configurability. This tool was created to address those issues by:

- Providing native support for multiple output backends
- Offering a more predictable performance profile
- Being easy to configure and integrate in modern observability stacks

Performance comparison between Blackbox Exporter available [here](https://zookoo-stm.neryolab.com/benchmark/blackbox/REPORT-1000-URLS.md).

# Documentation

For more information on how to use Zookoo, please refer to the [documentation](https://zookoo-stm.neryolab.com), or in this repository under the `documentation` -> `docs` folder.

# Installation

## With docker (recommended)

Use the docker image `neryo/zookoo:latest`

## Manual installation

Use the binary

### From the repository

**You need cargo and rust installed**

```bash
git clone <the repo> zookoo

cd zookoo
cargo build # build the project accordingly to your system

# It create a binary in the /target folder
```

# Usage

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

  forward_to = [exporter.otlp.default]
}

exporter "otlp" "default" {
  url = "http://localhost:4317"
  tls_insecure = true
}

```

For more examples, please refer to the [documentation](https://zookoo-stm.neryolab.com).

# Contribution

If you want to contribute to this project, feel free to open issues or submit pull requests on GitHub.

More details in the [contribution guide](CONTRIBUTING.md).
