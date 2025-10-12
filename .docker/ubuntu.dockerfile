FROM rust:latest AS builder

WORKDIR /usr/src/app

# Copy source code
COPY . .

# Build the application in release mode
RUN cargo build --release

FROM ubuntu:latest

# Set user and group with no rights
RUN groupadd -r appgroup && useradd -r -g appgroup appuser

# Copy binary from builder
COPY --from=builder /usr/src/app/target/release/zookoo /usr/local/bin/zookoo

# Set permissions
RUN chown appuser:appgroup /usr/local/bin/zookoo
RUN chmod +x /usr/local/bin/zookoo

ENV PATH="/usr/local/bin:${PATH}"

ENTRYPOINT ["/usr/local/bin/zookoo"]

# Set a non-privileged user
USER appuser:appgroup

CMD [ "zookoo", "--config", "/etc/zookoo/config.toml" ]