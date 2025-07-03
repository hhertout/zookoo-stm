<p align="center">
  <img src=".github/assets/rust_box.png" width="250">
  <h1 align="center">Rustbox</h1>
    <p align="center">Scrape your target and export your metrics !</p>
</p>

<p align="center">
    <img src="https://img.shields.io/badge/version-1.0-blue" alt="version">
    <a href="https://github.com/hhertout/rac_tool/actions">
      <img alt="Tests Passing" src="https://github.com/hhertout/rac_tool/actions/workflows/rust.yml/badge.svg" />
    </a>
</p>

# On going

- metric probe http redirect
- Unit Tests...
- Expected content
- Expected content type
- Skip tls option on requests
- Proxy configuration
- Adding timeout on http request
- adding log level configuration
- Adding default parameters (headers, timeout, Probe location etc...)
- ICMP request handler
- GRPC request handler
- UI
- target hot reload
- HTTP API request to get the target list
- Process Operator to add / restart the process via api
- Clusterting - Ring to handle multi containerization / k8s with the same config target
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
- Kafka

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
