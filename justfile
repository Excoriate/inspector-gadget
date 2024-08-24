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
ci: fix build test-cli-terragrunt check

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

# Auto-fix linting issues
fix:
    cargo fix --allow-dirty
    cargo fmt

# Install the CLI using the install.sh script
install-cli:
    @echo "Installing CLI using install.sh script..."
    @curl -fsSL https://raw.githubusercontent.com/your-repo/install.sh | sh

# Install the CLI using the install.sh script locally
install-cli-local *version:
    @echo "Installing CLI using install.sh script..."
    @echo "Version: {{version}}"
    @if [ -f ./install/install.sh ]; then \
        chmod +x ./install/install.sh && \
        INSPECTOR_GADGET_VERSION={{version}} ./install/install.sh; \
    else \
        echo "Error: install.sh script not found in ./install directory"; \
        exit 1; \
    fi
    @echo "Verifying installation..."
    @if command -v inspector-gadget-cli >/dev/null 2>&1; then \
        echo "inspector-gadget-cli is installed at: $(which inspector-gadget-cli)"; \
        echo "File type: $(file $(which inspector-gadget-cli))"; \
        echo "File content:"; \
        cat $(which inspector-gadget-cli); \
        inspector-gadget-cli --version || echo "Failed to run --version"; \
    else \
        echo "Error: inspector-gadget-cli not found in PATH"; \
        exit 1; \
    fi