#!/usr/bin/env bash
set -euo pipefail

CAST_FILE="demo.cast"
GIF_FILE="demo.gif"
OUTPUT_DIR="assets/screenshots"
OUTPUT_PATH="$OUTPUT_DIR/$GIF_FILE"

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
