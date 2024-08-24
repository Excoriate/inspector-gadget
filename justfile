# List all available recipes
default:
    @just --list

# Run all tests
test:
    cargo test

# Run tests with verbose output
test-verbose:
    cargo test -- --nocapture

# Run clippy linter
lint:
    cargo clippy -- -D warnings

# Check code formatting
format-check:
    cargo fmt -- --check

# Format code
format:
    cargo fmt

# Build the project
build:
    cargo build

# Build the project in release mode
build-release:
    cargo build --release

# Run the CLI with provided arguments
run *args:
    cargo run -- {{args}}

# Clean the project
clean:
    cargo clean

# Run all checks (tests, lint, format)
check: test lint format-check

# Publish to crates.io (use with caution)
publish:
    cargo publish

# Test CLI functionality by downloading terragrunt docs and deleting the generated file
test-cli-terragrunt:
    @echo "Testing CLI with Terragrunt docs..."
    @just run https://terragrunt.gruntwork.io/docs/features/keep-your-remote-state-configuration-dry --show-links --output-format=txt --output-file=terragrunt-docs-links
    @test -f terragrunt-docs-links || (echo "File not created" && exit 1)
    @rm terragrunt-docs-links
    @echo "CLI test completed successfully"

# Run CI checks including compilation, functional test, and code quality checks
ci: build test-cli-terragrunt check

# Format code and apply fixes
format-fix:
    cargo fmt

# Build Docker image, run CLI, and execute CI tests
docker-ci:
    @echo "Building Docker image..."
    docker build -t inspector-cli .
    @echo "Running CLI in Docker container..."
    docker run --rm inspector-cli --help
    @echo "Running CI tests in Docker container..."
    docker run --rm -v $(pwd):/usr/src/inspector-cli -w /usr/src/inspector-cli rust:1.67 sh -c "apt-get update && apt-get install -y just && cargo test && cargo clippy -- -D warnings && cargo fmt -- --check && just test-cli-terragrunt"