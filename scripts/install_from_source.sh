#!/bin/bash

set -e

# Default version is latest
VERSION=${1:-latest}

# Temporary directory for cloning and building
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

echo "Installing Inspector Gadget CLI version ${VERSION} from source..."

# Clone the repository
git clone --depth 1 --branch "$VERSION" https://github.com/Excoriate/inspector-gadget-cli.git "$TMP_DIR"

# Navigate to the cloned directory
cd "$TMP_DIR"

# Build the project
cargo build --release

# Install the binary
sudo mv target/release/inspector-gadget /usr/local/bin/

echo "Inspector Gadget CLI installed successfully!"
echo "You can now use 'inspector-gadget' command."

# Verify installation
if inspector-gadget --version; then
    echo "Installation verified successfully."
else
    echo "Installation verification failed. Please check your PATH."
fi