#!/bin/bash

set -e

VERSION=${INSPECTOR_GADGET_VERSION:-latest}
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

if [ "$ARCH" = "x86_64" ]; then
    ARCH="amd64"
elif [ "$ARCH" = "aarch64" ]; then
    ARCH="arm64"
fi

# Updated binary name format based on the confirmed release
BINARY_NAME="inspector-gadget-cli-${OS}-${ARCH}"
if [ "$OS" = "windows" ]; then
    BINARY_NAME="${BINARY_NAME}.exe"
fi

BINARY_URL="https://github.com/Excoriate/inspector-gadget-cli/releases/download/${VERSION}/${BINARY_NAME}"

echo "Downloading Inspector Gadget CLI version ${VERSION} for ${OS}_${ARCH}..."
echo "URL: ${BINARY_URL}"

if ! curl -L -o inspector-gadget-cli "${BINARY_URL}"; then
    echo "Error: Failed to download the binary"
    exit 1
fi

echo "Verifying download..."
if ! [ -s inspector-gadget-cli ]; then
    echo "Error: Downloaded file is empty or does not exist"
    echo "Content of the file:"
    cat inspector-gadget-cli
    exit 1
fi

echo "File size: $(wc -c < inspector-gadget-cli) bytes"
echo "File type: $(file inspector-gadget-cli)"

echo "Installing Inspector Gadget CLI..."
chmod +x inspector-gadget-cli
if ! sudo mv inspector-gadget-cli /usr/local/bin/; then
    echo "Error: Failed to move the binary to /usr/local/bin/"
    exit 1
fi

echo "Inspector Gadget CLI installed successfully in /usr/local/bin"
echo "Verifying installation..."
if inspector-gadget-cli --version; then
    echo "Inspector Gadget CLI installed successfully!"
else
    echo "Error: Installation verification failed. Please check your PATH and try running 'inspector-gadget-cli --version' manually."
fi