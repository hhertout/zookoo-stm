# syntax=docker/dockerfile:1
FROM rust:latest AS builder

WORKDIR /usr/src/app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    git \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy everything and build (simple & reliable)
COPY . .

# Clean any stale build artifacts and build fresh
RUN cargo clean && cargo build --release

FROM ubuntu:24.04

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3t64 \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user (use existing GID/UID if already present)
RUN groupadd -g 1000 appuser 2>/dev/null || true && \
    useradd -r -u 1000 -g 1000 appuser 2>/dev/null || true && \
    chown -R 1000:1000 /app

# Copy the binary from builder
COPY --from=builder /usr/src/app/target/release/zookoo /app/zookoo
RUN chown 1000:1000 /app/zookoo

# Switch to non-root user
USER 1000

# Run the application
CMD ["/app/zookoo"]