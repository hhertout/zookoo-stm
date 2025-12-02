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

# Copy manifests first for better cache utilization
COPY Cargo.toml Cargo.lock ./
COPY configuration/Cargo.toml configuration/Cargo.toml
COPY discovery/Cargo.toml discovery/Cargo.toml
COPY engine/Cargo.toml engine/Cargo.toml
COPY exporter/Cargo.toml exporter/Cargo.toml
COPY probe/Cargo.toml probe/Cargo.toml

# Create dummy source files to build dependencies
RUN mkdir -p src configuration/src discovery/src engine/src exporter/src probe/src && \
    echo "fn main() {}" > src/main.rs && \
    echo "" > configuration/src/lib.rs && \
    echo "" > discovery/src/lib.rs && \
    echo "" > engine/src/lib.rs && \
    echo "" > exporter/src/lib.rs && \
    echo "" > probe/src/lib.rs

# Build dependencies only (cached layer)
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/src/app/target \
    cargo build --release

# Copy actual source code
COPY . .

# Touch source files to invalidate cache and rebuild
RUN touch src/main.rs

# Build the application
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/src/app/target \
    cargo build --release && \
    cp target/release/zookoo /usr/local/bin/zookoo

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
COPY --from=builder /usr/local/bin/zookoo /app/zookoo
RUN chown 1000:1000 /app/zookoo

# Switch to non-root user
USER 1000

# Run the application
CMD ["/app/zookoo"]