#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT_PATH="${1:-$ROOT_DIR/assets/screenshots/generated-fullscreen.png}"
DELAY="${DELAY:-6}"

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "This script currently supports macOS (uses screencapture)."
  echo "Capture manually on your platform and place it under assets/screenshots/."
  exit 1
fi

mkdir -p "$(dirname "$OUT_PATH")"

cat <<'MSG'
Preparing fullscreen screenshot capture.

1) Open your terminal fullscreen and run atmos-tui, for example:
   cargo run --release -- Stockholm
2) Leave that terminal visible.
3) Return here and let the countdown finish.
MSG

for ((i = DELAY; i >= 1; i--)); do
  printf "\rCapturing in %2ds..." "$i"
  sleep 1
done
printf "\n"

screencapture -x "$OUT_PATH"
echo "Saved screenshot: $OUT_PATH"
