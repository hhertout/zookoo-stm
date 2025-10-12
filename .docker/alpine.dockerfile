FROM rust:latest AS builder

WORKDIR /usr/src/app

# Copy source code
COPY . .

# Build the application in release mode
RUN cargo build --release

FROM alpine:latest

# Set user and group with no rights
RUN addgroup -S appgroup && adduser -S appuser -G appgroup

# Copy binary from builder
COPY --from=builder /usr/src/app/target/release/zookoo /usr/local/bin/zookoo

# Set permissions

RUN chown appuser:appgroup /usr/local/bin/zookoo
RUN chmod +x /usr/local/bin/zookoo

RUN echo "export PATH=\$PATH:/usr/local/bin" >> /home/appuser/.profile

ENTRYPOINT ["/usr/local/bin/zookoo"]

USER appuser:appgroup



CMD [ "zookoo", "--config", "/etc/zookoo/config.toml" ]