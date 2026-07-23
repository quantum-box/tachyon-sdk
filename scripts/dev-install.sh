#!/bin/sh
# Durable local-dev install of the tachyon CLI from source — PLT-2636.
#
# WHY: building from source and then symlinking
#   ~/.local/bin/tachyon -> <repo>/cli/target/release/tachyon
# breaks the CLI when target/ is removed (disk cleanup / cargo clean). On 2026-07-18
# this silently killed the `tachyon` command, taking down the CEO-escalation path
# (Slack @mention) and tachyon-browser. This script builds from source and installs a
# REAL FILE copy into ~/.local/bin (target-independent, survives target cleanup) —
# never a symlink into target/.
#
# For release installs use scripts/install.sh (downloads a release artifact). This script
# is for local development against unreleased source.
#
# Usage: sh scripts/dev-install.sh

set -e

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

# Keep build temp off the shared /tmp tmpfs (root CLAUDE.md).
export TMPDIR="${TMPDIR:-$HOME/buildtmp}"
mkdir -p "$TMPDIR" 2>/dev/null || true

echo "Building tachyon (release) from source..."
cargo build --release --bin tachyon

# Locate the built binary (workspace or crate-local target layout).
BIN=""
for candidate in "$ROOT/target/release/tachyon" "$ROOT/cli/target/release/tachyon"; do
  if [ -x "$candidate" ]; then
    BIN="$candidate"
    break
  fi
done
if [ -z "$BIN" ]; then
  echo "build succeeded but tachyon binary was not found in target/release" >&2
  exit 1
fi

DEST_DIR="${HOME}/.local/bin"
mkdir -p "$DEST_DIR"

# Real-file copy (NOT a symlink) so the CLI survives target/ removal (PLT-2636).
install -m 755 "$BIN" "${DEST_DIR}/tachyon"

echo "Installed durable tachyon to ${DEST_DIR}/tachyon ($("${DEST_DIR}/tachyon" --version 2>/dev/null || echo '??'))"

case ":${PATH}:" in
  *":${DEST_DIR}:"*) ;;
  *)
    echo ""
    echo "  NOTE: ${DEST_DIR} is not in your PATH. Add:"
    echo "    export PATH=\"${DEST_DIR}:\$PATH\""
    ;;
esac

echo ""
echo "Do NOT symlink ${DEST_DIR}/tachyon -> ${ROOT}/cli/target/release/tachyon."
echo "A symlink into target/ breaks on cleanup and silently takes down the CLI (PLT-2636)."
