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
