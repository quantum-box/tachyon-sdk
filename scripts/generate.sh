#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
SPEC_FILE="$ROOT_DIR/openapi.json"

if [ ! -f "$SPEC_FILE" ]; then
  echo "Error: openapi.json not found at $SPEC_FILE"
  exit 1
fi

# Check if openapi-generator-cli is available
if command -v openapi-generator-cli &> /dev/null; then
  GENERATOR="openapi-generator-cli"
elif command -v openapi-generator &> /dev/null; then
  GENERATOR="openapi-generator"
elif [ -f /usr/local/bin/openapi-generator-cli ]; then
  GENERATOR="/usr/local/bin/openapi-generator-cli"
else
  echo "Error: openapi-generator-cli not found."
  echo "Install: npm install @openapitools/openapi-generator-cli -g"
  echo "   or:   brew install openapi-generator"
  exit 1
fi

echo "Using generator: $GENERATOR"
echo "Spec file: $SPEC_FILE"
echo ""

# --- Rust SDK ---
echo "=== Generating Rust SDK ==="
$GENERATOR generate \
  -g rust \
  -i "$SPEC_FILE" \
  -o "$ROOT_DIR/rust" \
  --additional-properties=packageName=tachyon-sdk,packageVersion=0.1.0,library=reqwest \
  --skip-validate-spec

echo ""

# --- TypeScript SDK ---
echo "=== Generating TypeScript SDK ==="
$GENERATOR generate \
  -g typescript-fetch \
  -i "$SPEC_FILE" \
  -o "$ROOT_DIR/typescript" \
  --additional-properties=npmName=@tachyon/sdk,npmVersion=0.1.0,typescriptThreePlus=true,supportsES6=true \
  --skip-validate-spec

echo ""

# --- Python SDK ---
echo "=== Generating Python SDK ==="
$GENERATOR generate \
  -g python \
  -i "$SPEC_FILE" \
  -o "$ROOT_DIR/python" \
  --additional-properties=packageName=tachyon_sdk,packageVersion=0.1.0,projectName=tachyon-sdk \
  --skip-validate-spec

echo ""
echo "=== SDK generation complete ==="
echo "Generated:"
echo "  - rust/"
echo "  - typescript/"
echo "  - python/"
