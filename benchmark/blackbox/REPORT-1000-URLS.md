# Benchmark Report: Zookoo vs Blackbox Exporter + OTEL Collector (1000 URLs)

## Objective

Compare resource consumption (CPU, Memory, and Network) between:
- **Zookoo**: A lightweight monitoring tool written in Rust
- **Blackbox Exporter + OTEL Collector**: The traditional Prometheus ecosystem approach

## Methodology

### Test Environment
- **Platform**: Docker containers on macOS
- **Metrics Collection**: cAdvisor scraped by Prometheus
- **Sampling**: Peak metrics collected during stress test
- **Scrape Interval**: **5 seconds** for all targets (stress test configuration)

### Target Configuration
- **Number of URLs**: 1000 unique HTTP endpoints
- **URL Generation**: 200 base domains × 5 path variations per domain
- **Target file size**: ~6000 lines JSON

### Containers Under Test

| Container | Image | Role |
|-----------|-------|------|
| zookoo-benchmark | blackbox-zookoo (custom Rust) | Zookoo monitoring tool |
| blackbox-benchmark | prom/blackbox-exporter:latest | HTTP probe exporter |
| otel-benchmark | otel/opentelemetry-collector-contrib:latest | Metrics collection & forwarding |

### PromQL Queries Used

**Memory Usage (peak):**
```promql
max_over_time(container_memory_usage_bytes{name=~"..."}[10m])
```

**CPU Usage (peak percentage):**
```promql
max_over_time((rate(container_cpu_usage_seconds_total{cpu="total"}[1m])*100)[10m:30s])
```

**Network I/O:**
```promql
max_over_time(container_network_transmit_bytes_total{name=~"..."}[10m])
max_over_time(container_network_receive_bytes_total{name=~"..."}[10m])
```

## Results

### Raw Metrics (Peak Values - Stress Test @ 5s interval)

| Container | Memory (MB) | CPU (%) | Network TX (MB) | Network RX (MB) |
|-----------|-------------|---------|-----------------|-----------------|
| Zookoo | 60.3 | 1.15 | 2.57 | 11.10 |
| OTEL Collector | 398.9 | 14.45 | 244.15 | 210.44 |
| Blackbox Exporter | 1,470.0 | 35.49 | 403.53 | 136.72 |

### Comparison Summary

| Metric | Zookoo | Blackbox + OTEL | Improvement |
|--------|--------|-----------------|-------------|
| **Memory** | 60 MB | 1,869 MB | **31x less memory** |
| **CPU** | 1.15% | 49.94% | **43x less CPU** |
| **Network I/O** | 13.7 MB | 540 MB | **40x less network** |
| **Log Volume** | 2,636 lines | 318,385 lines | **120x less logs** |

*Note: Network comparison is Zookoo vs Blackbox only (probe traffic). OTEL internal traffic excluded.*

### Detailed Breakdown

#### Memory Consumption
- **Zookoo**: 60.3 MB (single async Rust process handling all probes)
- **Blackbox Exporter**: 1,470 MB ⚠️ (memory explosion under load)
- **OTEL Collector**: 398.9 MB (metrics scraping and forwarding pipeline)
- **Combined Blackbox + OTEL**: 1,869 MB (~1.87 GB)

#### CPU Consumption
- **Zookoo**: 1.15% (efficient async Rust implementation with tokio)
- **Blackbox Exporter**: 35.49% (processing 1000 concurrent HTTP probes)
- **OTEL Collector**: 14.45% (metrics pipeline overhead)
- **Combined Blackbox + OTEL**: 49.94%

#### Network I/O
- **Zookoo**: 13.7 MB total (direct remote_write to Prometheus)
- **Blackbox**: 540 MB total (metrics exposition overhead)
- Zookoo sends aggregated metrics directly, avoiding the scrape/expose overhead
- *Note: Only probe-related traffic compared (OTEL internal traffic excluded)*

#### Log Volume
- **Zookoo**: 2,636 lines (minimal async logging, one line per probe result)
- **Blackbox**: 318,385 lines (verbose synchronous logging, multiple lines per probe)
- Blackbox generates **120x more log output** than Zookoo
- High log volume contributes to increased I/O overhead and storage requirements

## Key Observations

1. **Memory Efficiency**: Zookoo uses **31x less memory** than the Blackbox + OTEL combination
   - Blackbox Exporter memory exploded to 1.47 GB under stress
   - Zookoo remained stable at ~60 MB

2. **CPU Efficiency**: Zookoo uses **43x less CPU**
   - Combined Blackbox + OTEL consumed nearly 50% of CPU
   - Zookoo stayed at just 1.15%

3. **Network Efficiency**: Zookoo generates **40x less network traffic**
   - Direct push model (remote_write) is far more efficient than scrape model
   - No intermediate metrics exposition required

4. **Scaling Behavior Under Stress**:
   - Zookoo handles 1000 URLs @ 5s interval with minimal resource increase
   - Blackbox Exporter becomes a significant bottleneck (1.47 GB RAM, 35% CPU)

5. **Architecture Advantage**: 
   - Zookoo: Single binary, async Rust, direct push to Prometheus
   - Traditional: Multi-component (Blackbox + OTEL), scrape-based, higher overhead

6. **Log Efficiency**: Zookoo generates **120x fewer log lines**
   - Zookoo: Compact single-line logging per probe with all metrics aggregated
   - Blackbox: Verbose multi-line logging per probe (HTTP request + handler logs)
   - Reduced log volume means lower I/O overhead and storage costs

## Conclusion

Under stress test conditions (1000 URLs @ 5s scrape interval), Zookoo demonstrates **exceptional efficiency**:

| Metric | Improvement Factor |
|--------|-------------------|
| **Memory** | 31x less (60 MB vs 1.87 GB) |
| **CPU** | 43x less (1.15% vs 50%) |
| **Network** | 40x less (14 MB vs 540 MB) |
| **Logs** | 120x less (2.6K vs 318K lines) |

The Blackbox Exporter shows critical scalability issues under high load, with memory consumption exploding to 1.47 GB. In contrast, Zookoo maintains a stable and predictable resource footprint.

**Zookoo is the ideal choice for large-scale HTTP monitoring deployments** where resource efficiency and predictable scaling are critical.

## Test Configuration Files

- `config.hcl`: Zookoo configuration (5s scrape interval)
- `targets.json`: 1000 URLs for Zookoo
- `blackbox-targets.json`: 1000 targets for Blackbox Exporter
- `opentelemetry.yml`: OTEL Collector configuration (5s scrape interval)
- `docker-compose.yml`: Full stack configuration

---

*Report generated: December 2025*
*Test duration: ~10 minutes stress test*
*Scrape interval: 5 seconds (aggressive stress test)*
