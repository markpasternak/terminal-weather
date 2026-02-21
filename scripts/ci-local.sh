#!/usr/bin/env bash
set -euo pipefail

WITH_CODACY=0
if [[ "${1:-}" == "--with-codacy" ]]; then
  WITH_CODACY=1
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
run_step "Run static-analysis gate" ./scripts/static-analysis-gate.sh
run_step "Run duplication analysis (advisory)" ./scripts/duplication-gate.sh
run_step "Run check" cargo check --all-targets --all-features
run_step "Run tests" cargo test --all --all-features
run_step "Build release" cargo build --release

if [[ "$WITH_CODACY" -eq 1 ]]; then
  run_step "Run codacy complexity parity gate" ./scripts/codacy-complexity-gate.sh
fi
