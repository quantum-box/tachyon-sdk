#!/bin/sh
# Install the Tachyon CLI
# Usage: curl -fsSL https://raw.githubusercontent.com/quantum-box/tachyon-sdk/main/scripts/install.sh | sh

set -e

REPO="quantum-box/tachyon-sdk"
BIN_NAME="tachyon"

# Detect OS
OS="$(uname -s)"
case "$OS" in
  Linux)  OS="linux" ;;
  Darwin) OS="darwin" ;;
  *)
    echo "Unsupported OS: $OS" >&2
    exit 1
    ;;
esac

# Detect architecture
ARCH="$(uname -m)"
case "$ARCH" in
  x86_64)          ARCH="x86_64" ;;
  aarch64 | arm64) ARCH="arm64" ;;
  *)
    echo "Unsupported architecture: $ARCH" >&2
    exit 1
    ;;
esac

ARTIFACT="${BIN_NAME}-${OS}-${ARCH}"

# Resolve the latest release asset via GitHub's /releases/latest/download/<asset>
# redirect. This avoids the GitHub REST API anonymous 60 req/hr/IP rate limit:
# the URL 302s to releases/download/<tag>/<asset>, no REST call required.
DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/${ARTIFACT}.tar.gz"

echo "Downloading ${BIN_NAME} (${OS}/${ARCH}) from latest release..."
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

curl -fsSL \
  -H "User-Agent: tachyon-sdk-installer/1.0" \
  "$DOWNLOAD_URL" \
  -o "${TMP_DIR}/${ARTIFACT}.tar.gz"

tar -xzf "${TMP_DIR}/${ARTIFACT}.tar.gz" -C "$TMP_DIR"

# Determine install location
if [ -w /usr/local/bin ]; then
  INSTALL_DIR="/usr/local/bin"
elif [ "$(id -u)" = "0" ]; then
  INSTALL_DIR="/usr/local/bin"
  mkdir -p "$INSTALL_DIR"
else
  INSTALL_DIR="${HOME}/.local/bin"
  mkdir -p "$INSTALL_DIR"
fi

install -m 755 "${TMP_DIR}/${BIN_NAME}" "${INSTALL_DIR}/${BIN_NAME}"

echo "Installed ${BIN_NAME} to ${INSTALL_DIR}/${BIN_NAME}"

# Warn if install dir is not in PATH
case ":${PATH}:" in
  *":${INSTALL_DIR}:"*) ;;
  *)
    echo ""
    echo "  NOTE: ${INSTALL_DIR} is not in your PATH."
    echo "  Add the following to your shell profile:"
    echo "    export PATH=\"${INSTALL_DIR}:\$PATH\""
    ;;
esac
