#!/bin/bash

set -e

# Function to print colorful messages
print_message() {
    local color=$1
    local message=$2
    echo -e "\033[${color}m${message}\033[0m"
}

# Function to handle errors
handle_error() {
    print_message "31" "❌ Error: $1"
    exit 1
}

VERSION=${INSPECTOR_GADGET_VERSION:-latest}
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

print_message "36" "🔍 Detecting system information..."
print_message "32" "  • Operating System: $OS"
print_message "32" "  • Architecture: $ARCH"

if [ "$ARCH" = "x86_64" ]; then
    ARCH="amd64"
elif [ "$ARCH" = "aarch64" ] || [ "$ARCH" = "arm64" ]; then
    ARCH="arm64"
else
    handle_error "Unsupported architecture: $ARCH"
fi

BINARY_NAME="inspector-gadget-${VERSION}-${ARCH}-${OS}"
if [ "$OS" = "darwin" ]; then
    BINARY_NAME="inspector-gadget-${VERSION}-${ARCH}-apple-darwin"
elif [ "$OS" = "windows" ]; then
    BINARY_NAME="${BINARY_NAME}.exe"
elif [ "$OS" = "linux" ]; then
    print_message "33" "ℹ️  Linux support added"
else
    handle_error "Unsupported operating system: $OS"
fi

RELEASE_URL="https://github.com/Excoriate/inspector-gadget-cli/releases/download/${VERSION}/${BINARY_NAME}.tar.gz"

print_message "36" "📥 Downloading Inspector Gadget CLI version ${VERSION} for ${OS}_${ARCH}..."
print_message "32" "  • URL: ${RELEASE_URL}"

if ! curl -L -o inspector-gadget.tar.gz "${RELEASE_URL}"; then
    handle_error "Failed to download the release"
fi

print_message "36" "📦 Extracting archive..."
if ! tar -xzf inspector-gadget.tar.gz; then
    handle_error "Failed to extract the archive"
fi

EXTRACTED_DIR=$(tar -tzf inspector-gadget.tar.gz | head -1 | cut -f1 -d"/")
BINARY_PATH="${EXTRACTED_DIR}/inspector-gadget"

print_message "36" "🛠️  Installing Inspector Gadget CLI..."
chmod +x "${BINARY_PATH}" || handle_error "Failed to set execute permissions"
if ! sudo mv "${BINARY_PATH}" /usr/local/bin/inspector-gadget; then
    handle_error "Failed to move the binary to /usr/local/bin/"
fi

print_message "36" "🧹 Cleaning up..."
rm -rf "${EXTRACTED_DIR}" inspector-gadget.tar.gz

print_message "32" "✅ Inspector Gadget CLI installed successfully in /usr/local/bin"
print_message "36" "🔍 Verifying installation..."
if inspector-gadget --help; then
    print_message "32" "🎉 Inspector Gadget CLI installed and verified successfully!"
else
    handle_error "Installation verification failed. Please check your PATH and try running 'inspector-gadget --version' manually."
fi