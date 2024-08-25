#!/bin/bash

set -e

VERSION=${1:-latest}
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

if [ "$ARCH" = "x86_64" ]; then
    ARCH="amd64"
elif [ "$ARCH" = "aarch64" ] || [ "$ARCH" = "arm64" ]; then
    ARCH="arm64"
fi

BINARY_NAME="inspector-gadget-${VERSION}-${ARCH}-${OS}"
if [ "$OS" = "darwin" ]; then
    BINARY_NAME="inspector-gadget-${VERSION}-${ARCH}-apple-darwin"
elif [ "$OS" = "windows" ]; then
    BINARY_NAME="${BINARY_NAME}.exe"
fi

RELEASE_URL="https://github.com/Excoriate/inspector-gadget-cli/releases/download/${VERSION}/${BINARY_NAME}.tar.gz"

echo "Downloading Inspector Gadget CLI version ${VERSION} for ${OS}_${ARCH}..."
echo "URL: ${RELEASE_URL}"

if ! curl -L -o inspector-gadget.tar.gz "${RELEASE_URL}"; then
    echo "Error: Failed to download the release"
    exit 1
fi

echo "Extracting archive..."
tar -xzf inspector-gadget.tar.gz

EXTRACTED_DIR=$(tar -tzf inspector-gadget.tar.gz | head -1 | cut -f1 -d"/")
BINARY_PATH="${EXTRACTED_DIR}/inspector-gadget"

echo "Installing Inspector Gadget CLI..."
chmod +x "${BINARY_PATH}"
if ! sudo mv "${BINARY_PATH}" /usr/local/bin/inspector-gadget; then
    echo "Error: Failed to move the binary to /usr/local/bin/"
    exit 1
fi

echo "Cleaning up..."
rm -rf "${EXTRACTED_DIR}" inspector-gadget.tar.gz

echo "Inspector Gadget CLI installed successfully in /usr/local/bin"
echo "Verifying installation..."
if inspector-gadget --help; then
    echo "Inspector Gadget CLI installed successfully!"
else
    echo "Error: Installation verification failed. Please check your PATH and try running 'inspector-gadget --version' manually."
fi