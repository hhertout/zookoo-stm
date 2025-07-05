---
sidebar_position: 2
---

# Deploy your collection layer and Grafana

**If you have already one, you can skip this section.**

Else, follow these steps to deploy your own Grafana instance.

## Collection Layer

### OpenTelemetry Collector

```yaml
# otel.yml
receivers:
  otlp:
    protocols:
      grpc:
        endpoint: 0.0.0.0:4317
      http:
        endpoint: 0.0.0.0:4318

exporters:
  debug:
    verbosity: detailed
  prometheusremotewrite:
    endpoint: http://mimir:9009/api/v1/push
    tls:
      insecure: true

processors:
  batch:
  tail_sampling:
    decision_wait: 30s
    policies:
      [
        {
          name: sample-erroring-traces,
          type: status_code,
          status_code: { status_codes: [ERROR] },
        },
        {
          name: sample-long-traces,
          type: latency,
          latency: { threshold_ms: 200 },
        },
      ]

service:
  pipelines:
    traces:
      receivers: [otlp]
      exporters: [debug]
    metrics:
      receivers: [otlp]
      processors: [batch]
      exporters: [debug, prometheusremotewrite]
    logs:
      receivers: [otlp]
      exporters: [debug]
```

## Grafana Stack

### Mimir configuration

```yaml
# mimir.yml
multitenancy_enabled: false
blocks_storage:
  backend: filesystem
  bucket_store:
    sync_dir: /tmp/mimir/tsdb-sync
  filesystem:
    dir: /tmp/mimir/data/tsdb
  tsdb:
    dir: /tmp/mimir/tsdb
compactor:
  data_dir: /tmp/mimir/compactor
  sharding_ring:
    kvstore:
      store: memberlist
distributor:
  ring:
    instance_addr: 127.0.0.1
    kvstore:
      store: memberlist
ingester:
  ring:
    instance_addr: 127.0.0.1
    kvstore:
      store: memberlist
    replication_factor: 1
ruler_storage:
  backend: filesystem
  filesystem:
    dir: /tmp/mimir/rules
server:
  http_listen_port: 9009
  log_level: info
store_gateway:
  sharding_ring:
    replication_factor: 1
limits:
  max_global_exemplars_per_user: 100000
  ingestion_rate: 30000
  past_grace_period: 720h
  creation_grace_period: 720h
  out_of_order_time_window: 720h
```

### Grafana stack docker compose

```yaml
# docker-compose.yml
services:
  otel:
    image: otel/opentelemetry-collector-contrib:latest
    volumes:
      - .otel.yml:/etc/otel/config.yaml
    ports:
      - "4317:4317"
      - "4318:4318"
    command: ["--config", "/etc/otel/config.yaml"]

  mimir:
    image: grafana/mimir:latest
    command:
      [
        "-ingester.native-histograms-ingestion-enabled=true",
        "-config.file=/etc/mimir.yaml",
      ]
    ports:
      - "9009:9009"
    volumes:
      - "./mimir.yml:/etc/mimir.yaml"

  grafana:
    image: grafana/grafana:latest
    environment:
      - GF_FEATURE_TOGGLES_ENABLE=flameGraph traceqlSearch correlations traceQLStreaming metricsSummary traceqlEditor traceToMetrics traceToProfiles datatrails
      - GF_INSTALL_PLUGINS=grafana-lokiexplore-app,grafana-exploretraces-app,grafana-pyroscope-app
      - GF_AUTH_ANONYMOUS_ENABLED=true
      - GF_AUTH_ANONYMOUS_ORG_ROLE=Admin
      - GF_AUTH_DISABLE_LOGIN_FORM=true
    volumes:
      - "grafana-data:/var/lib/grafana"
    ports:
      - "3000:3000"
    depends_on:
      - mimir
    user: root
```

### Run the stack

```bash
docker compose up -d
```

### Access Grafana

Open your browser and go to [http://localhost:3000](http://localhost:3000).

### Add the mimir data source

1. Go to **Configuration** > **Data Sources**.
2. Click on **Add data source**.
3. Select **Prometheus**.
4. Set the URL to `http://mimir:9009`.
5. Click on **Save & Test**.
