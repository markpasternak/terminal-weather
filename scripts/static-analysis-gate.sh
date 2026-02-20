#!/usr/bin/env bash
set -euo pipefail

if ! command -v rust-code-analysis-cli >/dev/null 2>&1; then
  echo "error: rust-code-analysis-cli is required (install with: cargo install --locked rust-code-analysis-cli --version 0.0.25)"
  exit 2
fi

if ! command -v jq >/dev/null 2>&1; then
  echo "error: jq is required (install with your package manager)"
  exit 2
fi

CYCLOMATIC_MAX="${TW_CYCLOMATIC_MAX:-20}"
COGNITIVE_MAX="${TW_COGNITIVE_MAX:-30}"
MI_MIN="${TW_MI_MIN:-30}"

tmp_dir="$(mktemp -d)"
metrics_tsv="$(mktemp)"
trap 'rm -rf "$tmp_dir" "$metrics_tsv"' EXIT

rust-code-analysis-cli -p src -p tests -m -F -O json --pr -o "$tmp_dir" >/dev/null

while IFS= read -r -d '' json_file; do
  jq -r --arg file "$json_file" --arg root "$tmp_dir/" '
    def nodes:
      . as $n
      | [ $n ] + (((($n.spaces // []) | map(nodes) | add) // []));
    nodes[]
    | select(.kind == "function")
    | [
        ($file | sub("^" + $root; "") | sub("\\.json$"; "")),
        .name,
        (.start_line // 0),
        (.metrics.cyclomatic.sum // 0),
        (.metrics.cognitive.sum // 0),
        (.metrics.mi.mi_original // 0)
      ]
    | @tsv
  ' "$json_file" >> "$metrics_tsv"
done < <(find "$tmp_dir" -name '*.json' -print0)

total_functions="$(wc -l < "$metrics_tsv" | tr -d ' ')"
cyc_violations="$(awk -F'\t' -v max="$CYCLOMATIC_MAX" '$4+0 >= max {count++} END {print count+0}' "$metrics_tsv")"
cog_violations="$(awk -F'\t' -v max="$COGNITIVE_MAX" '$5+0 >= max {count++} END {print count+0}' "$metrics_tsv")"
mi_violations="$(awk -F'\t' -v min="$MI_MIN" '$6+0 < min {count++} END {print count+0}' "$metrics_tsv")"

echo "Static analysis summary:"
echo "  total functions: ${total_functions}"
echo "  thresholds: cyclomatic < ${CYCLOMATIC_MAX}, cognitive < ${COGNITIVE_MAX}, MI >= ${MI_MIN}"
echo "  violations: cyclomatic=${cyc_violations}, cognitive=${cog_violations}, mi=${mi_violations}"

if [[ "$cyc_violations" -gt 0 || "$cog_violations" -gt 0 || "$mi_violations" -gt 0 ]]; then
  echo
  echo "Top violating functions:"
  awk -F'\t' -v cyc="$CYCLOMATIC_MAX" -v cog="$COGNITIVE_MAX" -v mi="$MI_MIN" '
    ($4+0 >= cyc) || ($5+0 >= cog) || ($6+0 < mi) { print }
  ' "$metrics_tsv" \
    | sort -t$'\t' -k4,4nr -k5,5nr -k6,6n \
    | head -20 \
    | awk -F'\t' '{
      printf("  - %s:%s %s (cyclomatic=%s, cognitive=%s, mi=%s)\n", $1, $3, $2, $4, $5, $6)
    }'
  exit 1
fi

echo "Static analysis gate passed."
