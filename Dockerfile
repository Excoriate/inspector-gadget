# Use a multi-stage build for a smaller final image
FROM rust:1.67-slim-buster as builder

# Create a new empty shell project
RUN USER=root cargo new --bin inspector-cli
WORKDIR /inspector-cli

# Copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# Cache dependencies
RUN cargo build --release
RUN rm src/*.rs

# Copy your source tree
COPY ./src ./src

# Build for release
RUN rm ./target/release/deps/inspector_cli*
RUN cargo build --release

# Final stage
FROM debian:buster-slim

# Install OpenSSL - required for HTTPS requests
RUN apt-get update && apt-get install -y openssl ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy the build artifact from the builder stage
COPY --from=builder /inspector-cli/target/release/inspector-cli /usr/local/bin/inspector-cli

# Set the entrypoint
ENTRYPOINT ["inspector-cli"]

# Metadata
LABEL org.opencontainers.image.source="https://github.com/yourusername/inspector-cli"
LABEL org.opencontainers.image.description="A CLI tool for inspecting and analyzing web links"
LABEL org.opencontainers.image.licenses="MIT"