#!/usr/bin/env bash
set -euo pipefail

if ! command -v cargo >/dev/null 2>&1; then
  echo "error: cargo is required"
  exit 2
fi

if ! cargo dupes --help >/dev/null 2>&1; then
  echo "error: cargo-dupes is required (install with: cargo install --locked cargo-dupes --version 0.2.1)"
  exit 2
fi

ANALYZE_PATH="${TW_DUPES_PATH:-.}"
EXCLUDE_PATTERN="${TW_DUPES_EXCLUDE:-target}"
STATS_PATH="${TW_DUPES_STATS_PATH:-target/duplication/dupes-stats.json}"
ENFORCE="${TW_DUPES_ENFORCE:-0}"
MAX_EXACT_PERCENT="${TW_DUPES_MAX_EXACT_PERCENT:-5.0}"
MAX_NEAR_PERCENT="${TW_DUPES_MAX_NEAR_PERCENT:-10.0}"

mkdir -p "$(dirname "$STATS_PATH")"

echo "Duplication analysis summary:"
echo "  path: ${ANALYZE_PATH}"
echo "  exclude: ${EXCLUDE_PATTERN}"
echo "  stats artifact: ${STATS_PATH}"
echo "  mode: $([[ "$ENFORCE" == "1" ]] && echo strict || echo advisory)"

echo
echo "Full duplication report:"
cargo dupes --path "$ANALYZE_PATH" --exclude "$EXCLUDE_PATTERN" --format text report

cargo dupes --path "$ANALYZE_PATH" --exclude "$EXCLUDE_PATTERN" --format json stats > "$STATS_PATH"

echo
echo "Duplication stats (json):"
cat "$STATS_PATH"

if [[ "$ENFORCE" == "1" ]]; then
  echo
  echo "Strict duplication check:"
  cargo dupes --path "$ANALYZE_PATH" --exclude "$EXCLUDE_PATTERN" check \
    --max-exact-percent "$MAX_EXACT_PERCENT" \
    --max-near-percent "$MAX_NEAR_PERCENT"
else
  echo
  echo "Advisory mode: duplication findings do not fail this gate."
  echo "Set TW_DUPES_ENFORCE=1 to enable strict thresholds."
fi
