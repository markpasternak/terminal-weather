#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CAST_FILE="$ROOT_DIR/demo.cast"
GIF_FILE="$ROOT_DIR/demo.gif"
OUTPUT_PATH="$ROOT_DIR/assets/screenshots/demo.gif"

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "Missing required command: $1" >&2
    exit 1
  fi
}

require_cmd cargo
require_cmd asciinema
require_cmd agg
require_cmd gifsicle

cd "$ROOT_DIR"

echo "==> Building release binary..."
cargo build --release

echo "==> Recording demo..."
clear
asciinema record "$CAST_FILE" --overwrite -c "clear && ./target/release/terminal-weather --demo"

echo "==> Converting to GIF..."
agg "$CAST_FILE" "$GIF_FILE" --idle-time-limit 2

echo "==> Optimizing GIF..."
gifsicle --optimize=3 --lossy=80 --resize-fit-width 900 -o "$GIF_FILE" "$GIF_FILE"

echo "==> Moving to $OUTPUT_PATH..."
mv "$GIF_FILE" "$OUTPUT_PATH"

echo "==> Cleaning up..."
rm -f "$CAST_FILE"

FINAL_SIZE=$(du -h "$OUTPUT_PATH" | cut -f1)
echo "==> Done! $OUTPUT_PATH ($FINAL_SIZE)"
