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

# Copy source code
COPY . .

# Build the application in release mode
RUN cargo build --release

FROM alpine:latest

WORKDIR /app

# Install runtime dependencies
RUN apk add --no-cache ca-certificates

# Create a non-root user
RUN addgroup -g 1000 appuser && \
    adduser -D -u 1000 -G appuser appuser && \
    chown -R appuser:appuser /app

# Copy the binary from builder
COPY --from=builder /usr/src/app/target/release/zookoo /app/
RUN chown appuser:appuser /app/zookoo

# Switch to non-root user
USER appuser

# Run the application
CMD ["/app/zookoo"]