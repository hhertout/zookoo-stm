# TO FIX

FROM rust:1-slim-bookworm AS builder

WORKDIR /usr/src/app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    git \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy source code
COPY . .

# Build the application in release mode
RUN cargo build --release

FROM ubuntu:24.04
WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN groupadd -g 1000 appuser && \
    useradd -r -u 1000 -g appuser appuser && \
    chown -R appuser:appuser /app

# Copy the binary from builder
COPY --from=builder /usr/src/app/target/release/zookoo /app/
RUN chown appuser:appuser /app/zookoo

# Switch to non-root user
USER appuser

# Run the application
CMD ["/app/zookoo"]