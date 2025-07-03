---
sidebar_position: 3
---

# Deploy with docker

The easiest way to test ZookooZookoo is to run it with docker

```bash
docker run neryo/zoukouzoukou:latest -v ./config.toml:/etc/rustbox/config.toml
```

or with a docker compose file

```yaml
services:
  zoukouzoukou:
    image: neryo/zoukouzoukou:latest
    volumes:
      - ./config.toml:/etc/rustbox/config.toml
    ports:
      - 12345:12345
```
