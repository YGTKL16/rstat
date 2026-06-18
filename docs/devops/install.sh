#!/bin/sh
# Shell installer for rstat (Linux & macOS)
# Run: curl -fsSL https://rstat.dev/install.sh | sh

set -eu

REPO="ygtkula/rstat"
BINARY="rstat-cli"
ALIAS="rstat"

case "$(uname -sm)" in
  "Darwin arm64")  TARGET="aarch64-apple-darwin" ;;
  "Darwin x86_64") TARGET="x86_64-apple-darwin" ;;
  "Linux aarch64") TARGET="aarch64-unknown-linux-musl" ;;
  "Linux x86_64")  TARGET="x86_64-unknown-linux-musl" ;;
  *) echo "Unsupported platform: $(uname -sm)"; exit 1 ;;
esac

VERSION=$(curl -s "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed 's/.*"v\([^"]*\)".*/\1/')
URL="https://github.com/${REPO}/releases/download/v${VERSION}/${BINARY}-v${VERSION}-${TARGET}.tar.gz"

echo "Downloading ${ALIAS} v${VERSION} for ${TARGET}..."
curl -sSfL "${URL}" | tar -xz -C /tmp

echo "Installing to /usr/local/bin/${ALIAS}..."
sudo install -m755 /tmp/${BINARY} /usr/local/bin/${ALIAS}

echo "✓ rstat installed successfully! Run 'rstat --help' to verify."
