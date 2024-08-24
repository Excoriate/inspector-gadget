# Build stage
FROM rust:1.67 as builder

WORKDIR /usr/src/inspector-cli
COPY . .

RUN cargo build --release

# Runtime stage
FROM debian:buster-slim

RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/inspector-cli/target/release/inspector-cli /usr/local/bin/inspector-cli

ENTRYPOINT ["inspector-cli"]

# Metadata
LABEL org.opencontainers.image.source="https://github.com/yourusername/inspector-cli"
LABEL org.opencontainers.image.description="A CLI tool for inspecting and analyzing web links"
LABEL org.opencontainers.image.licenses="MIT"