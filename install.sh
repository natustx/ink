#!/bin/sh
# ink installer — https://github.com/borghei/ink
# Usage: curl -fsSL https://raw.githubusercontent.com/borghei/ink/main/install.sh | sh

set -e

REPO="borghei/ink"
INSTALL_DIR="/usr/local/bin"

# Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Linux)   os="linux" ;;
  Darwin)  os="macos" ;;
  *)       echo "Unsupported OS: $OS"; exit 1 ;;
esac

case "$ARCH" in
  x86_64|amd64)   arch="amd64" ;;
  arm64|aarch64)   arch="arm64" ;;
  *)               echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac

BINARY="ink-${os}-${arch}"

# Get latest release tag
LATEST=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name"' | head -1 | cut -d'"' -f4)

if [ -z "$LATEST" ]; then
  echo "Could not determine latest release. Check https://github.com/$REPO/releases"
  exit 1
fi

URL="https://github.com/$REPO/releases/download/$LATEST/$BINARY"

echo "Downloading ink $LATEST for $os/$arch..."
TMPFILE=$(mktemp)
curl -fsSL "$URL" -o "$TMPFILE"
chmod +x "$TMPFILE"

# Install
if [ -w "$INSTALL_DIR" ]; then
  mv "$TMPFILE" "$INSTALL_DIR/ink"
else
  echo "Installing to $INSTALL_DIR (requires sudo)..."
  sudo mv "$TMPFILE" "$INSTALL_DIR/ink"
fi

echo "ink $LATEST installed to $INSTALL_DIR/ink"
echo "Run 'ink --help' to get started."
