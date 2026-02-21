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
EXCLUDE_TESTS="${TW_DUPES_EXCLUDE_TESTS:-1}"
MAX_EXACT_PERCENT="${TW_DUPES_MAX_EXACT_PERCENT:-5.0}"
MAX_NEAR_PERCENT="${TW_DUPES_MAX_NEAR_PERCENT:-10.0}"

mkdir -p "$(dirname "$STATS_PATH")"

if [[ "$EXCLUDE_TESTS" != "0" && "$EXCLUDE_TESTS" != "1" ]]; then
  echo "error: TW_DUPES_EXCLUDE_TESTS must be 0 or 1"
  exit 2
fi

dupes_args=(--path "$ANALYZE_PATH" --exclude "$EXCLUDE_PATTERN")
if [[ "$EXCLUDE_TESTS" == "1" ]]; then
  dupes_args+=(--exclude-tests)
fi

echo "Duplication analysis summary:"
echo "  path: ${ANALYZE_PATH}"
echo "  exclude: ${EXCLUDE_PATTERN}"
echo "  exclude test code: $([[ "$EXCLUDE_TESTS" == "1" ]] && echo yes || echo no)"
echo "  stats artifact: ${STATS_PATH}"
echo "  mode: $([[ "$ENFORCE" == "1" ]] && echo strict || echo advisory)"

echo
echo "Full duplication report:"
cargo dupes "${dupes_args[@]}" --format text report

cargo dupes "${dupes_args[@]}" --format json stats > "$STATS_PATH"

echo
echo "Duplication stats (json):"
cat "$STATS_PATH"

if [[ "$ENFORCE" == "1" ]]; then
  echo
  echo "Strict duplication check:"
  cargo dupes "${dupes_args[@]}" check \
    --max-exact-percent "$MAX_EXACT_PERCENT" \
    --max-near-percent "$MAX_NEAR_PERCENT"
else
  echo
  echo "Advisory mode: duplication findings do not fail this gate."
  echo "Set TW_DUPES_ENFORCE=1 to enable strict thresholds."
fi
