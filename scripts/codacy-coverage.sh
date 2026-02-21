#!/usr/bin/env bash
set -euo pipefail

UPLOAD="${TW_CODACY_UPLOAD:-0}"
REPORT_PATH="${TW_CODACY_REPORT_PATH:-coverage/lcov.info}"
LANGUAGE="${TW_CODACY_LANGUAGE:-rust}"

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

if ! rustup component list --installed | grep -q '^llvm-tools-preview'; then
  echo "info: installing llvm-tools-preview"
  rustup component add llvm-tools-preview
fi

mkdir -p "$(dirname "$REPORT_PATH")"

echo "Generating LCOV report at ${REPORT_PATH}"
cargo llvm-cov --workspace --all-features --lcov --output-path "$REPORT_PATH"

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
