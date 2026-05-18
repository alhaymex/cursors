#!/usr/bin/env bash
set -euo pipefail

APP_NAME="cursors"
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
PREFIX="${PREFIX:-$HOME/.local}"
BIN_DIR="$PREFIX/bin"
TARGET="$BIN_DIR/$APP_NAME"

cd "$SCRIPT_DIR"

if ! command -v cargo >/dev/null 2>&1; then
  echo "error: cargo is required to build $APP_NAME" >&2
  exit 1
fi

echo "Building $APP_NAME..."
cargo build --release

mkdir -p "$BIN_DIR"
install -m 0755 "target/release/$APP_NAME" "$TARGET"

echo "Installed $APP_NAME to $TARGET"

case ":$PATH:" in
  *":$BIN_DIR:"*) ;;
  *)
    echo
    echo "warning: $BIN_DIR is not on PATH"
    echo "Add this to your shell config:"
    echo "  export PATH=\"$BIN_DIR:\$PATH\""
    ;;
esac

echo
echo "Run it with:"
echo "  $APP_NAME"
