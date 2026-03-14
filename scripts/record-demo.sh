#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CAST_FILE="$ROOT_DIR/demo.cast"
GIF_FILE="$ROOT_DIR/demo_intermediate.gif"
WEBP_FILE="$ROOT_DIR/demo.webp"
OUTPUT_PATH="$ROOT_DIR/assets/screenshots/demo.webp"

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "Missing required command: $1" >&2
    exit 1
  fi
}

require_cmd cargo
require_cmd asciinema
require_cmd agg
require_cmd ffmpeg

cd "$ROOT_DIR"

echo "==> Building release binary..."
cargo build --release

echo "==> Recording demo..."
clear
asciinema record "$CAST_FILE" --overwrite -c "clear && ./target/release/terminal-weather --demo"

echo "==> Converting to intermediate GIF..."
agg "$CAST_FILE" "$GIF_FILE" --idle-time-limit 2

echo "==> Encoding to WebP..."
ffmpeg -y -i "$GIF_FILE" \
  -vf "fps=15,scale=838:-1:flags=lanczos" \
  -loop 0 -quality 80 \
  "$WEBP_FILE"

echo "==> Moving to $OUTPUT_PATH..."
mv "$WEBP_FILE" "$OUTPUT_PATH"

echo "==> Cleaning up..."
rm -f "$CAST_FILE" "$GIF_FILE"

FINAL_SIZE=$(du -h "$OUTPUT_PATH" | cut -f1)
echo "==> Done! $OUTPUT_PATH ($FINAL_SIZE)"
