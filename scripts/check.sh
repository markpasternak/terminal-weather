#!/usr/bin/env bash
set -euo pipefail

WITH_FULL=0
if [[ "${1:-}" == "--full" ]]; then
  WITH_FULL=1
fi

run_step() {
  local label="$1"
  shift
  echo
  echo "==> ${label}"
  "$@"
}

run_step "Check formatting" cargo fmt --all -- --check
run_step "Run clippy" cargo clippy --all-targets --all-features -- -D warnings
run_step "Run clippy pedantic gate" cargo clippy --all-targets --all-features -- \
  -D warnings \
  -D clippy::pedantic \
  -D clippy::if_same_then_else \
  -D clippy::match_same_arms \
  -D clippy::branches_sharing_code
run_step "Run complexity gate" ./scripts/complexity-gate.sh
run_step "Run duplication analysis (advisory)" ./scripts/duplication-gate.sh
run_step "Run check" cargo check --all-targets --all-features
run_step "Run tests" cargo test --all --all-features
run_step "Build release" cargo build --release

if [[ "$WITH_FULL" -eq 1 ]]; then
  run_step "Run complexity audit" ./scripts/complexity-audit.sh

  if [[ -n "${CODACY_PROJECT_TOKEN:-}" || -n "${CODACY_API_TOKEN:-}" ]]; then
    run_step "Generate and upload coverage report" env TW_CODACY_UPLOAD=1 ./scripts/coverage.sh
  else
    run_step "Generate coverage report (upload skipped: no token set)" ./scripts/coverage.sh
  fi
fi
