#!/usr/bin/env bash
set -euo pipefail

UPLOAD="${TW_CODACY_UPLOAD:-0}"
REPORT_PATH="${TW_CODACY_REPORT_PATH:-coverage/lcov.info}"
LANGUAGE="${TW_CODACY_LANGUAGE:-rust}"
BRANCH_COVERAGE="${TW_CODACY_BRANCH_COVERAGE:-1}"
FAIL_ON_THRESHOLD="${TW_CODACY_FAIL_ON_THRESHOLD:-1}"
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
THRESHOLDS_FILE="${TW_CODACY_THRESHOLDS_FILE:-${SCRIPT_DIR}/coverage-thresholds.env}"

COVERAGE_LF=0
COVERAGE_LH=0
COVERAGE_FNF=0
COVERAGE_FNH=0
COVERAGE_BRF=0
COVERAGE_BRH=0
COVERAGE_LINE_PCT=-1
COVERAGE_FUNCTION_PCT=-1
COVERAGE_BRANCH_PCT=-1

MIN_LINE=85
MIN_FUNCTION=85
MIN_BRANCH=75

collect_coverage_metrics() {
  local report_path="$1"

  read -r COVERAGE_LF COVERAGE_LH COVERAGE_FNF COVERAGE_FNH COVERAGE_BRF COVERAGE_BRH COVERAGE_LINE_PCT COVERAGE_FUNCTION_PCT COVERAGE_BRANCH_PCT < <(
    awk '
      BEGIN {
        lf = 0; lh = 0;
        fnf = 0; fnh = 0;
        brf = 0; brh = 0;
      }
      /^LF:/ { lf += substr($0, 4) }
      /^LH:/ { lh += substr($0, 4) }
      /^FNF:/ { fnf += substr($0, 5) }
      /^FNH:/ { fnh += substr($0, 5) }
      /^BRF:/ { brf += substr($0, 5) }
      /^BRH:/ { brh += substr($0, 5) }
      END {
        line_pct = (lf > 0) ? (lh / lf) * 100 : -1;
        function_pct = (fnf > 0) ? (fnh / fnf) * 100 : -1;
        branch_pct = (brf > 0) ? (brh / brf) * 100 : -1;
        printf "%d %d %d %d %d %d %.6f %.6f %.6f\n", lf, lh, fnf, fnh, brf, brh, line_pct, function_pct, branch_pct;
      }
    ' "$report_path"
  )
}

validate_threshold_value() {
  local name="$1"
  local value="$2"

  if ! [[ "$value" =~ ^[0-9]+([.][0-9]+)?$ ]]; then
    echo "error: ${name} must be numeric, got '${value}'"
    exit 2
  fi

  if ! awk -v value="$value" 'BEGIN { exit (value >= 0 && value <= 100) ? 0 : 1 }'; then
    echo "error: ${name} must be between 0 and 100, got '${value}'"
    exit 2
  fi
}

validate_toggle_value() {
  local name="$1"
  local value="$2"

  if [[ "$value" != "0" && "$value" != "1" ]]; then
    echo "error: ${name} must be 0 or 1, got '${value}'"
    exit 2
  fi
}

load_coverage_thresholds() {
  if [[ -f "$THRESHOLDS_FILE" ]]; then
    # shellcheck disable=SC1090
    source "$THRESHOLDS_FILE"
  fi

  MIN_LINE="${TW_CODACY_MIN_LINE:-85}"
  MIN_FUNCTION="${TW_CODACY_MIN_FUNCTION:-85}"
  MIN_BRANCH="${TW_CODACY_MIN_BRANCH:-75}"

  validate_threshold_value "TW_CODACY_MIN_LINE" "$MIN_LINE"
  validate_threshold_value "TW_CODACY_MIN_FUNCTION" "$MIN_FUNCTION"
  validate_threshold_value "TW_CODACY_MIN_BRANCH" "$MIN_BRANCH"
  validate_toggle_value "TW_CODACY_FAIL_ON_THRESHOLD" "$FAIL_ON_THRESHOLD"
}

is_less_than() {
  local lhs="$1"
  local rhs="$2"

  awk -v lhs="$lhs" -v rhs="$rhs" 'BEGIN { exit (lhs < rhs) ? 0 : 1 }'
}

print_coverage_summary() {
  local report_path="$1"

  echo "Coverage summary from ${report_path}:"
  if [[ "$COVERAGE_LF" -gt 0 ]]; then
    printf "  line coverage: %.2f%% (%d/%d)\n" "$COVERAGE_LINE_PCT" "$COVERAGE_LH" "$COVERAGE_LF"
  else
    echo "  line coverage: n/a"
  fi

  if [[ "$COVERAGE_FNF" -gt 0 ]]; then
    printf "  function coverage: %.2f%% (%d/%d)\n" "$COVERAGE_FUNCTION_PCT" "$COVERAGE_FNH" "$COVERAGE_FNF"
  else
    echo "  function coverage: n/a"
  fi

  if [[ "$COVERAGE_BRF" -gt 0 ]]; then
    printf "  branch coverage: %.2f%% (%d/%d)\n" "$COVERAGE_BRANCH_PCT" "$COVERAGE_BRH" "$COVERAGE_BRF"
  else
    echo "  branch coverage: n/a (no BRF/BRH data)"
  fi
}

enforce_coverage_thresholds() {
  local failed=0
  local issue_prefix="error"

  if [[ "$FAIL_ON_THRESHOLD" -eq 0 ]]; then
    issue_prefix="warning"
  fi

  echo "Coverage thresholds:"
  printf "  line coverage: >= %.2f%%\n" "$MIN_LINE"
  printf "  function coverage: >= %.2f%%\n" "$MIN_FUNCTION"
  printf "  branch coverage: >= %.2f%%\n" "$MIN_BRANCH"

  if [[ "$COVERAGE_LF" -eq 0 ]]; then
    echo "${issue_prefix}: line coverage data is missing from ${REPORT_PATH}"
    failed=1
  elif is_less_than "$COVERAGE_LINE_PCT" "$MIN_LINE"; then
    printf "%s: line coverage %.2f%% is below threshold %.2f%%\n" "$issue_prefix" "$COVERAGE_LINE_PCT" "$MIN_LINE"
    failed=1
  fi

  if [[ "$COVERAGE_FNF" -eq 0 ]]; then
    echo "${issue_prefix}: function coverage data is missing from ${REPORT_PATH}"
    failed=1
  elif is_less_than "$COVERAGE_FUNCTION_PCT" "$MIN_FUNCTION"; then
    printf "%s: function coverage %.2f%% is below threshold %.2f%%\n" "$issue_prefix" "$COVERAGE_FUNCTION_PCT" "$MIN_FUNCTION"
    failed=1
  fi

  if [[ "$COVERAGE_BRF" -eq 0 ]]; then
    echo "${issue_prefix}: branch coverage data is missing from ${REPORT_PATH}"
    failed=1
  elif is_less_than "$COVERAGE_BRANCH_PCT" "$MIN_BRANCH"; then
    printf "%s: branch coverage %.2f%% is below threshold %.2f%%\n" "$issue_prefix" "$COVERAGE_BRANCH_PCT" "$MIN_BRANCH"
    failed=1
  fi

  if [[ "$failed" -eq 1 ]]; then
    if [[ "$FAIL_ON_THRESHOLD" -eq 1 ]]; then
      echo "error: coverage gate failed"
      exit 1
    fi

    echo "warning: coverage gate failed (non-blocking)"
    return
  fi

  echo "Coverage gate passed."
}

validate_codacy_auth() {
  local has_project_token=0
  local has_api_token=0

  if [[ -n "${CODACY_PROJECT_TOKEN:-}" ]]; then
    has_project_token=1
  fi

  if [[ -n "${CODACY_API_TOKEN:-}" ]]; then
    has_api_token=1
  fi

  if [[ "$has_project_token" -eq 1 ]]; then
    return 0
  fi

  if [[ "$has_api_token" -eq 1 ]]; then
    local missing=0

    if [[ -z "${CODACY_ORGANIZATION_PROVIDER:-}" ]]; then
      echo "error: CODACY_ORGANIZATION_PROVIDER is required when using CODACY_API_TOKEN"
      missing=1
    fi
    if [[ -z "${CODACY_USERNAME:-}" ]]; then
      echo "error: CODACY_USERNAME is required when using CODACY_API_TOKEN"
      missing=1
    fi
    if [[ -z "${CODACY_PROJECT_NAME:-}" ]]; then
      echo "error: CODACY_PROJECT_NAME is required when using CODACY_API_TOKEN"
      missing=1
    fi

    if [[ "$missing" -eq 1 ]]; then
      return 1
    fi

    return 0
  fi

  echo "error: set CODACY_PROJECT_TOKEN, or CODACY_API_TOKEN with CODACY_ORGANIZATION_PROVIDER/CODACY_USERNAME/CODACY_PROJECT_NAME"
  return 1
}

print_install_help() {
  echo "error: cargo-llvm-cov is required (install with: cargo install --locked cargo-llvm-cov)"
}

if ! cargo llvm-cov --version >/dev/null 2>&1; then
  print_install_help
  exit 2
fi

load_coverage_thresholds

mkdir -p "$(dirname "$REPORT_PATH")"

echo "Generating LCOV report at ${REPORT_PATH}"
if [[ "$BRANCH_COVERAGE" -eq 1 ]]; then
  if ! command -v rustup >/dev/null 2>&1; then
    echo "error: rustup is required for branch coverage mode"
    exit 2
  fi

  if ! rustup toolchain list | grep -q '^nightly'; then
    echo "info: installing nightly toolchain for branch coverage"
    rustup toolchain install nightly --profile minimal
  fi

  echo "info: ensuring llvm-tools-preview for nightly toolchain"
  rustup component add llvm-tools-preview --toolchain nightly

  cargo +nightly llvm-cov --workspace --all-features --lcov --branch --output-path "$REPORT_PATH"
else
  if ! rustup component list --installed | grep -q '^llvm-tools-preview'; then
    echo "info: installing llvm-tools-preview"
    rustup component add llvm-tools-preview
  fi

  cargo llvm-cov --workspace --all-features --lcov --output-path "$REPORT_PATH"
fi

collect_coverage_metrics "$REPORT_PATH"
print_coverage_summary "$REPORT_PATH"
enforce_coverage_thresholds

if [[ "$UPLOAD" -eq 1 ]]; then
  if ! validate_codacy_auth; then
    exit 2
  fi

  if ! command -v curl >/dev/null 2>&1; then
    echo "error: curl is required for Codacy upload"
    exit 2
  fi

  tmp_script="$(mktemp)"
  trap 'rm -f "$tmp_script"' EXIT

  echo "Uploading coverage report to Codacy"
  curl -LSsf https://coverage.codacy.com/get.sh -o "$tmp_script"
  bash "$tmp_script" report -r "$REPORT_PATH" --language "$LANGUAGE"
fi
