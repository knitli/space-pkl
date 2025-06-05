#!/bin/bash
set -e

PKL_VERSION=$1

if [ -z "$PKL_VERSION" ]; then
    echo "Usage: $0 <pkl-version>"
    exit 1
fi

echo "Installing Pkl CLI version $PKL_VERSION"

# Detect platform
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case $ARCH in
    x86_64)
        ARCH="amd64"
        ;;
    arm64|aarch64)
        ARCH="aarch64"
        ;;
    *)
        echo "Unsupported architecture: $ARCH"
        exit 1
        ;;
esac

case $OS in
    linux)
        FILE_EXT="tar.gz"
        ;;
    darwin)
        OS="macos"
        FILE_EXT="tar.gz"
        ;;
    mingw*|msys*|cygwin*)
        OS="windows"
        FILE_EXT="zip"
        ;;
    *)
        echo "Unsupported OS: $OS"
        exit 1
        ;;
esac

# Create installation directory
INSTALL_DIR="$HOME/.moon/tools/pkl/$PKL_VERSION"
mkdir -p "$INSTALL_DIR"

# Download URL
ARCHIVE_NAME="pkl-cli-${OS}-${ARCH}.${FILE_EXT}"
DOWNLOAD_URL="https://github.com/apple/pkl/releases/download/${PKL_VERSION}/${ARCHIVE_NAME}"

echo "Downloading from: $DOWNLOAD_URL"

# Download and extract
if command -v curl >/dev/null 2>&1; then
    curl -L -o "/tmp/${ARCHIVE_NAME}" "$DOWNLOAD_URL"
elif command -v wget >/dev/null 2>&1; then
    wget -O "/tmp/${ARCHIVE_NAME}" "$DOWNLOAD_URL"
else
    echo "Neither curl nor wget found. Cannot download."
    exit 1
fi

# Extract archive
cd "$INSTALL_DIR"
if [ "$FILE_EXT" = "tar.gz" ]; then
    tar -xzf "/tmp/${ARCHIVE_NAME}"
elif [ "$FILE_EXT" = "zip" ]; then
    unzip "/tmp/${ARCHIVE_NAME}"
fi

# Make executable (Unix-like systems)
if [ "$OS" != "windows" ]; then
    chmod +x "$INSTALL_DIR/pkl"
fi

# Add to PATH for current session
if [ "$OS" = "windows" ]; then
    export PATH="$INSTALL_DIR:$PATH"
    echo "$INSTALL_DIR" >> $GITHUB_PATH
else
    export PATH="$INSTALL_DIR:$PATH"
    echo "$INSTALL_DIR" >> $GITHUB_PATH
fi

# Verify installation
if [ "$OS" = "windows" ]; then
    PKL_BINARY="$INSTALL_DIR/pkl.exe"
else
    PKL_BINARY="$INSTALL_DIR/pkl"
fi

if [ -x "$PKL_BINARY" ]; then
    echo "✅ Pkl CLI $PKL_VERSION installed successfully"
    "$PKL_BINARY" --version
else
    echo "❌ Pkl CLI installation failed"
    exit 1
fi

# Clean up
rm -f "/tmp/${ARCHIVE_NAME}"
