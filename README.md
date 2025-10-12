<p align="center">
  <img src=".github/assets/zookoo.png" width="250">

  <h1 align="center">Zookoo</h1>
    <p align="center">Scrape your target and export your metrics !</p>
</p>

<p align="center">
    <img src="https://img.shields.io/badge/version-0.0.1-blue" alt="version">
    <a href="https://github.com/hhertout/rac_tool/actions">
      <img alt="Tests Passing" src="https://github.com/hhertout/rac_tool/actions/workflows/rust.yml/badge.svg" />
    </a>
</p>

# TODO List

- Global:

  - Check config function (main.rs)
  - check histogram metrics -> doesn't work properly (to confirm)
  - Unit Tests...
  - Configuration Hot Module Reload
  - Process Operator to add / restart the process via api
  - UI
  - Load testing (k6 ?) & benchmarking
  - Clusterting - Ring to handle multi containerization / k8s with the same config target

- Defaults:

  - Headers
  - Timeout

- Targets:

  - Labels on ICMP targets
  - Probe HTTP redirect & follow redirect
  - Expected content
  - Expected content type
  - Proxy configuration
  - GRPC request handler
  - DNS test and expiration date of a dns name
  - Skip tls option on requests

- Discovery:

  - Discovery API http targets

- Exporter:

  - Kafka exporter
  - InfluxDB exporter
  - MongoDB exporter
  - TimescaleDB / PostgreSQL exporter
  - Referential building

# Presentation

This project provides an alternative to Blackbox, offering improved performance and greater configurability.

It is built as a synthetic monitoring tool (STM) that allows you to scrape targets and send data to your chosen service — all powered by Rust’s blazingly fast performance.

**Scraping target** :

- Http
- Https
- ICMP

**Exporter** :

- self (via a dedicated endpoint)
- Open telemetry (http/protobuf & grpc)

# Why this project?

In large-scale environments, we've found Blackbox Exporter to present limitations in terms of performance and configurability. This tool was created to address those issues by:

- Providing native support for multiple output backends
- Offering a more predictable performance profile
- Being easy to configure and integrate in modern observability stacks

# Documentation

For more information on how to use Zookoo, please refer to the [documentation](). (coming soon)

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

# Configuration

# Contribution
