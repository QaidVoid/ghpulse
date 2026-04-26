#!/usr/bin/env bash
# install.sh — downloads ghpulse binary from GitHub Releases
set -euo pipefail

VERSION="${GHPULSE_VERSION:-latest}"
ARCH="$(uname -m)"

case "$ARCH" in
  x86_64|amd64)  TARGET_ARCH="x86_64" ;;
  aarch64|arm64) TARGET_ARCH="aarch64" ;;
  *) echo "Unsupported architecture: $ARCH" >&2; exit 1 ;;
esac

TARGET="${TARGET_ARCH}-unknown-linux-musl"

if [ "$VERSION" = "latest" ]; then
  DOWNLOAD_URL="https://github.com/qaidvoid/ghpulse/releases/latest/download/ghpulse-${TARGET}.tar.gz"
else
  DOWNLOAD_URL="https://github.com/qaidvoid/ghpulse/releases/download/${VERSION}/ghpulse-${TARGET}.tar.gz"
fi

echo "Downloading ghpulse ($TARGET) from $DOWNLOAD_URL"
curl -fsSL "$DOWNLOAD_URL" | tar xz -C /usr/local/bin ghpulse
chmod +x /usr/local/bin/ghpulse
ghpulse --version
