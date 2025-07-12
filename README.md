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

# On going

- Discovery API http targets
- histogram metrics doesn't work properly
- Labels on ICMP targets
- Kafka exporter
- InfluxDB exporter
- MongoDB exporter
- TimescaleDB / PostgreSQL exporter
- DNS test and expiration date of a dns name
- metric probe http redirect
- Unit Tests...
- Expected content
- Expected content type
- Skip tls option on requests
- Proxy configuration
- Adding default parameters (headers, timeout, Probe location etc...)
- GRPC request handler
- UI
- Massive load testing
- target hot reload
- HTTP API request to get the target list
- Process Operator to add / restart the process via api
- Clusterting - Ring to handle multi containerization / k8s with the same config target
- Referential building

Caution : Icmp scraping requires root privileges to run properly, as it uses the ping command under the hood.
Run in sudo...

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

# Installation

## With docker

This is the recommended way.

Use the docker image `neryo/rustbox:latest`

## Manual installation

Use the binary

### From the repository

**You need cargo and rust installed**

```bash
git clone <the repo> redbox

cd redbox
cargo build # build the project accordingly to your system

# It create a binary in the /target folder
```

# Usage

# Configuration

# Contribution
