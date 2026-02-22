#!/usr/bin/env bash
set -euo pipefail

LINE_LIMIT="${TW_FILE_LENGTH_LIMIT:-500}"

declare -a search_paths=()
for path in src tests; do
  if [[ -d "$path" ]]; then
    search_paths+=("$path")
  fi
done

if [[ ${#search_paths[@]} -eq 0 ]]; then
  echo "File length audit summary:"
  echo "  scope: src/ + tests/ (*.rs)"
  echo "  threshold: > ${LINE_LIMIT} lines"
  echo "  scanned files: 0"
  echo "  files over threshold: 0"
  exit 0
fi

tmp_metrics="$(mktemp)"
trap 'rm -f "$tmp_metrics"' EXIT

while IFS= read -r -d '' file; do
  line_count="$(wc -l < "$file" | tr -d ' ')"
  printf '%s\t%s\n' "$line_count" "$file" >> "$tmp_metrics"
done < <(find "${search_paths[@]}" -type f -name '*.rs' -print0 | sort -z)

scanned_files="$(wc -l < "$tmp_metrics" | tr -d ' ')"
over_limit_count="$(awk -F'\t' -v limit="$LINE_LIMIT" '$1+0 > limit {count++} END {print count+0}' "$tmp_metrics")"

echo "File length audit summary:"
echo "  scope: src/ + tests/ (*.rs)"
echo "  threshold: > ${LINE_LIMIT} lines"
echo "  scanned files: ${scanned_files}"
echo "  files over threshold: ${over_limit_count}"

if [[ "$over_limit_count" -gt 0 ]]; then
  echo
  echo "Files over ${LINE_LIMIT} lines:"
  awk -F'\t' -v limit="$LINE_LIMIT" '$1+0 > limit {print}' "$tmp_metrics" \
    | sort -t$'\t' -k1,1nr -k2,2 \
    | awk -F'\t' '{ printf("  - %s (%s lines)\n", $2, $1) }'
fi
