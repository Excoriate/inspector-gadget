# Build stage
FROM rust:1.67-alpine as builder

# Install build dependencies
RUN apk add --no-cache musl-dev

# Create a new empty shell project
WORKDIR /inspector-cli

# Copy over your manifests
COPY Cargo.toml Cargo.lock ./

# Cache dependencies
RUN mkdir src && \
    echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs && \
    cargo build --release && \
    rm -f target/release/deps/inspector_cli*

# Copy your source tree
COPY ./src ./src

# Build for release
RUN cargo build --release

# Final stage
FROM alpine:3.14

# Install runtime dependencies
RUN apk add --no-cache libgcc

# Copy the build artifact from the builder stage
COPY --from=builder /inspector-cli/target/release/inspector-cli /usr/local/bin/

# Set the entrypoint
ENTRYPOINT ["inspector-cli"]

# Metadata
LABEL org.opencontainers.image.source="https://github.com/yourusername/inspector-cli"
LABEL org.opencontainers.image.description="A CLI tool for inspecting and analyzing web links"
LABEL org.opencontainers.image.licenses="MIT"