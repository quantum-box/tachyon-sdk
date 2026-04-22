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

# Fetch latest release tag from GitHub API.
# Some environments (corporate proxies, strict egress, rate-limited IPs) 403
# the API when the request lacks a User-Agent / Accept header, so we set both.
# If GITHUB_TOKEN is provided, use it to lift the anonymous rate limit.
API_URL="https://api.github.com/repos/${REPO}/releases/latest"
AUTH_HEADER=""
if [ -n "${GITHUB_TOKEN:-}" ]; then
  AUTH_HEADER="Authorization: Bearer ${GITHUB_TOKEN}"
fi

LATEST_TAG="$(curl -fsSL \
  -H "User-Agent: tachyon-sdk-installer/1.0" \
  -H "Accept: application/vnd.github.v3+json" \
  ${AUTH_HEADER:+-H "$AUTH_HEADER"} \
  "$API_URL" \
  | grep '"tag_name"' \
  | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')"

if [ -z "$LATEST_TAG" ]; then
  echo "Failed to fetch latest release tag from ${API_URL}." >&2
  echo "If you are hitting a GitHub API rate limit or 403, retry with:" >&2
  echo "  curl -fsSL https://raw.githubusercontent.com/${REPO}/main/scripts/install.sh | GITHUB_TOKEN=<token> sh" >&2
  exit 1
fi

DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${LATEST_TAG}/${ARTIFACT}.tar.gz"

echo "Downloading ${BIN_NAME} ${LATEST_TAG} (${OS}/${ARCH})..."
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

curl -fsSL "$DOWNLOAD_URL" -o "${TMP_DIR}/${ARTIFACT}.tar.gz"
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
