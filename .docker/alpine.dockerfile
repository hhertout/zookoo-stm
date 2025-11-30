# syntax=docker/dockerfile:1
FROM rust:alpine3.22 AS builder

WORKDIR /usr/src/app

# Install build dependencies including static OpenSSL libraries
RUN apk add --no-cache \
    build-base \
    musl-dev \
    openssl-dev \
    openssl-libs-static \
    pkgconfig \
    git \
    ca-certificates

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

FROM alpine:latest

WORKDIR /app

# Install runtime dependencies
RUN apk add --no-cache ca-certificates

# Create a non-root user
RUN addgroup -g 1000 appuser && \
    adduser -D -u 1000 -G appuser appuser && \
    chown -R appuser:appuser /app

# Copy the binary from builder
COPY --from=builder /usr/local/bin/zookoo /app/zookoo
RUN chown appuser:appuser /app/zookoo

# Switch to non-root user
USER appuser

# Run the application
CMD ["/app/zookoo"]