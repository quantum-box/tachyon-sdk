#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
APPS_ROOT="$(cd "$ROOT_DIR/.." && pwd)"
SPEC_FILE="${STOREKIT_OPENAPI_SPEC:-$APPS_ROOT/apps/bakuure-api/bakuure.openapi.yaml}"
OUTPUT_DIR="${STOREKIT_TYPESCRIPT_OUT:-$ROOT_DIR/typescript-storekit}"

if [ ! -f "$SPEC_FILE" ]; then
  echo "Error: StoreKit OpenAPI spec not found at $SPEC_FILE"
  echo "Run bakuure_codegen first, or set STOREKIT_OPENAPI_SPEC."
  exit 1
fi

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
echo "StoreKit spec: $SPEC_FILE"
echo "Output: $OUTPUT_DIR"

$GENERATOR generate \
  -g typescript-fetch \
  -i "$SPEC_FILE" \
  -o "$OUTPUT_DIR" \
  --additional-properties=npmName=@tachyon/storekit,npmVersion=0.1.0,typescriptThreePlus=true,supportsES6=true \
  --global-property=apis,models,supportingFiles \
  --skip-validate-spec

echo "StoreKit TypeScript SDK generated."
