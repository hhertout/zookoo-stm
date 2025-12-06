---
sidebar_position: 2
---

# Deploy with docker

The easiest way to test Zookoo is to run it with docker

```bash
docker run neryo/zookoo:latest -v ./config.toml:/etc/zookoo/config.hcl
```

or with a docker compose file

```yaml
services:
  zoukouzoukou:
    image: neryo/zookoo:latest
    volumes:
      - ./config.hcl:/etc/zookoo/config.hcl
    ports:
      - 12345:12345
```
