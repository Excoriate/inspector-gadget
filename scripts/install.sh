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

# Update the BINARY_NAME construction
BINARY_NAME="inspector-gadget-${VERSION}-x86_64-apple-darwin"

RELEASE_URL="https://github.com/Excoriate/inspector-gadget-cli/releases/download/${VERSION}/${BINARY_NAME}.tar.gz"

print_message "36" "📥 Downloading Inspector Gadget CLI version ${VERSION} for ${OS}_${ARCH}..."
print_message "32" "  • URL: ${RELEASE_URL}"

# Download with better error checking
if ! curl -L -o inspector-gadget.tar.gz "${RELEASE_URL}" --fail --silent --show-error; then
    handle_error "Failed to download the release. Please check the URL and your internet connection."
fi

# Check if the downloaded file is empty or too small
if [ ! -s inspector-gadget.tar.gz ] || [ "$(wc -c < inspector-gadget.tar.gz)" -lt 1000 ]; then
    handle_error "Downloaded file is empty or too small. Possible invalid URL or GitHub rate limiting."
fi

print_message "36" "📦 Extracting archive..."
if ! tar -xzf inspector-gadget.tar.gz; then
    handle_error "Failed to extract the archive. The downloaded file may be corrupted or not a valid tar.gz archive."
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