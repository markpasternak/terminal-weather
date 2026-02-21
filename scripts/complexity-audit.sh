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

FILE_NLOC_MEDIUM_MAX="${TW_FILE_NLOC_MEDIUM_MAX:-500}"
FILE_NLOC_CRITICAL_MAX="${TW_FILE_NLOC_CRITICAL_MAX:-1000}"
FUNCTION_NLOC_MEDIUM_MAX="${TW_FUNCTION_NLOC_MEDIUM_MAX:-50}"
FUNCTION_NLOC_CRITICAL_MAX="${TW_FUNCTION_NLOC_CRITICAL_MAX:-100}"
CYCLOMATIC_MEDIUM_MAX="${TW_CYCLOMATIC_MEDIUM_MAX:-8}"
CYCLOMATIC_CRITICAL_MAX="${TW_CYCLOMATIC_CRITICAL_MAX:-12}"
PARAM_MEDIUM_MAX="${TW_PARAM_MEDIUM_MAX:-8}"
PARAM_CRITICAL_MAX="${TW_PARAM_CRITICAL_MAX:-12}"
FAIL_ON_CRITICAL="${TW_COMPLEXITY_FAIL_ON_CRITICAL:-1}"
FAIL_ON_MEDIUM="${TW_COMPLEXITY_FAIL_ON_MEDIUM:-0}"

tmp_dir="$(mktemp -d)"
file_metrics_tsv="$(mktemp)"
function_metrics_tsv="$(mktemp)"
trap 'rm -rf "$tmp_dir" "$file_metrics_tsv" "$function_metrics_tsv"' EXIT

rust-code-analysis-cli -p src -p tests -m -F -O json --pr -o "$tmp_dir" >/dev/null

while IFS= read -r -d '' json_file; do
  jq -r --arg file "$json_file" --arg root "$tmp_dir/" '
    [
      ($file | sub("^" + $root; "") | sub("\\.json$"; "")),
      (.metrics.loc.ploc // 0)
    ]
    | @tsv
  ' "$json_file" >> "$file_metrics_tsv"

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
        (.metrics.loc.ploc // 0),
        (.metrics.cyclomatic.sum // 0),
        (.metrics.nargs.total_functions // 0)
      ]
    | @tsv
  ' "$json_file" >> "$function_metrics_tsv"
done < <(find "$tmp_dir" -name '*.json' -print0)

total_files="$(wc -l < "$file_metrics_tsv" | tr -d ' ')"
total_functions="$(wc -l < "$function_metrics_tsv" | tr -d ' ')"

file_critical_violations="$(awk -F'\t' -v max="$FILE_NLOC_CRITICAL_MAX" '$2+0 > max {count++} END {print count+0}' "$file_metrics_tsv")"
file_medium_violations="$(awk -F'\t' -v max="$FILE_NLOC_MEDIUM_MAX" '$2+0 > max {count++} END {print count+0}' "$file_metrics_tsv")"

function_nloc_critical_violations="$(awk -F'\t' -v max="$FUNCTION_NLOC_CRITICAL_MAX" '$4+0 > max {count++} END {print count+0}' "$function_metrics_tsv")"
function_nloc_medium_violations="$(awk -F'\t' -v max="$FUNCTION_NLOC_MEDIUM_MAX" '$4+0 > max {count++} END {print count+0}' "$function_metrics_tsv")"

cyclomatic_critical_violations="$(awk -F'\t' -v max="$CYCLOMATIC_CRITICAL_MAX" '$5+0 > max {count++} END {print count+0}' "$function_metrics_tsv")"
cyclomatic_medium_violations="$(awk -F'\t' -v max="$CYCLOMATIC_MEDIUM_MAX" '$5+0 > max {count++} END {print count+0}' "$function_metrics_tsv")"

param_critical_violations="$(awk -F'\t' -v max="$PARAM_CRITICAL_MAX" '$6+0 > max {count++} END {print count+0}' "$function_metrics_tsv")"
param_medium_violations="$(awk -F'\t' -v max="$PARAM_MEDIUM_MAX" '$6+0 > max {count++} END {print count+0}' "$function_metrics_tsv")"

critical_total=$((file_critical_violations + function_nloc_critical_violations + cyclomatic_critical_violations + param_critical_violations))
medium_total=$((file_medium_violations + function_nloc_medium_violations + cyclomatic_medium_violations + param_medium_violations))

echo "Complexity audit summary:"
echo "  scope: src/ + tests/"
echo "  analyzed: ${total_files} files, ${total_functions} function blocks"
echo "  critical thresholds: file NLOC <= ${FILE_NLOC_CRITICAL_MAX}, function NLOC <= ${FUNCTION_NLOC_CRITICAL_MAX}, cyclomatic <= ${CYCLOMATIC_CRITICAL_MAX}, params <= ${PARAM_CRITICAL_MAX}"
echo "  medium thresholds: file NLOC <= ${FILE_NLOC_MEDIUM_MAX}, function NLOC <= ${FUNCTION_NLOC_MEDIUM_MAX}, cyclomatic <= ${CYCLOMATIC_MEDIUM_MAX}, params <= ${PARAM_MEDIUM_MAX}"
echo "  critical violations: file_nloc=${file_critical_violations}, function_nloc=${function_nloc_critical_violations}, cyclomatic=${cyclomatic_critical_violations}, params=${param_critical_violations}"
echo "  medium violations: file_nloc=${file_medium_violations}, function_nloc=${function_nloc_medium_violations}, cyclomatic=${cyclomatic_medium_violations}, params=${param_medium_violations}"

if [[ "$file_critical_violations" -gt 0 ]]; then
  echo
  echo "Top critical file NLOC violations:"
  awk -F'\t' -v max="$FILE_NLOC_CRITICAL_MAX" '$2+0 > max {print}' "$file_metrics_tsv" \
    | sort -t$'\t' -k2,2nr \
    | head -20 \
    | awk -F'\t' '{ printf("  - %s (nloc=%s)\n", $1, $2) }'
fi

if [[ "$function_nloc_critical_violations" -gt 0 ]]; then
  echo
  echo "Top critical function NLOC violations:"
  awk -F'\t' -v max="$FUNCTION_NLOC_CRITICAL_MAX" '$4+0 > max {print}' "$function_metrics_tsv" \
    | sort -t$'\t' -k4,4nr \
    | head -20 \
    | awk -F'\t' '{ printf("  - %s:%s %s (nloc=%s)\n", $1, $3, $2, $4) }'
fi

if [[ "$cyclomatic_critical_violations" -gt 0 ]]; then
  echo
  echo "Top critical cyclomatic violations:"
  awk -F'\t' -v max="$CYCLOMATIC_CRITICAL_MAX" '$5+0 > max {print}' "$function_metrics_tsv" \
    | sort -t$'\t' -k5,5nr \
    | head -20 \
    | awk -F'\t' '{ printf("  - %s:%s %s (cyclomatic=%s)\n", $1, $3, $2, $5) }'
fi

if [[ "$param_critical_violations" -gt 0 ]]; then
  echo
  echo "Top critical parameter-count violations:"
  awk -F'\t' -v max="$PARAM_CRITICAL_MAX" '$6+0 > max {print}' "$function_metrics_tsv" \
    | sort -t$'\t' -k6,6nr \
    | head -20 \
    | awk -F'\t' '{ printf("  - %s:%s %s (params=%s)\n", $1, $3, $2, $6) }'
fi

if [[ "$critical_total" -gt 0 && "$FAIL_ON_CRITICAL" -eq 1 ]]; then
  echo
  echo "Complexity audit gate failed on critical thresholds."
  exit 1
fi

if [[ "$medium_total" -gt 0 && "$FAIL_ON_MEDIUM" -eq 1 ]]; then
  echo
  echo "Complexity audit gate failed on medium thresholds."
  exit 1
fi

echo "Complexity audit gate passed for configured fail policy."
