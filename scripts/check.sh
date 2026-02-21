#!/usr/bin/env bash
# Unified quality gate with structured output.
# Default: quiet mode — shows step progress and final report.
# Pass --verbose to see full tool output.
set -uo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"

# ── flags ─────────────────────────────────────────────────────
VERBOSE=0
if [[ "${1:-}" == "--verbose" || "${1:-}" == "-v" ]]; then
  VERBOSE=1
fi

# ── result arrays ─────────────────────────────────────────────
declare -a step_names=()
declare -a step_kinds=()    # REQUIRED | RECOMMENDED
declare -a step_results=()  # PASS | FAIL | WARN | SKIP
declare -a step_details=()

record() {
  step_names+=("$1")
  step_kinds+=("$2")
  step_results+=("$3")
  step_details+=("${4:-}")
}

# Detail extractors set this to "WARN" when the command succeeds
# but the output reveals concerning metrics.
_detail_warn=""

# ── detail extractors ─────────────────────────────────────────

detail_formatting() {
  local output_file="$1" rc="$2"
  _detail_warn=""
  if [[ "$rc" -eq 0 ]]; then
    echo "all files formatted"
  else
    local count
    count=$(grep -c '^Diff in' "$output_file" 2>/dev/null) || count=0
    echo "${count} file(s) need formatting"
  fi
}

detail_clippy() {
  local output_file="$1" rc="$2"
  _detail_warn=""
  if [[ "$rc" -eq 0 ]]; then
    echo "0 warnings"
  else
    local warns errs
    warns=$(grep -c '^ *warning\[' "$output_file" 2>/dev/null) || warns=0
    errs=$(grep -c '^ *error\[' "$output_file" 2>/dev/null) || errs=0
    echo "${warns} warning(s), ${errs} error(s)"
  fi
}

detail_complexity_gate() {
  local output_file="$1" rc="$2"
  _detail_warn=""
  if [[ "$rc" -eq 0 ]]; then
    local funcs
    funcs=$(grep -oE 'total functions: [0-9]+' "$output_file" | grep -oE '[0-9]+' || echo "?")
    echo "${funcs} functions, all within thresholds"
  else
    local line
    line=$(grep 'violations:' "$output_file" | tail -1)
    echo "${line:-threshold violations found}"
  fi
}

detail_cargo_check() {
  local output_file="$1" rc="$2"
  _detail_warn=""
  if [[ "$rc" -eq 0 ]]; then
    echo "all targets type-check"
  else
    local errs
    errs=$(grep -c '^error' "$output_file" 2>/dev/null) || errs=0
    echo "${errs} error(s)"
  fi
}

detail_tests() {
  local output_file="$1" rc="$2"
  _detail_warn=""
  if [[ "$rc" -eq 0 ]]; then
    local total
    total=$(grep -oE '[0-9]+ passed' "$output_file" | awk '{s+=$1} END {print s+0}')
    if [[ "$total" -gt 0 ]]; then
      echo "${total} passed"
    else
      echo "all tests passed"
    fi
  else
    local fails
    fails=$(grep -E '^\s+---- .* ----$' "$output_file" 2>/dev/null | head -3 | sed 's/^.*---- //;s/ ----$//' | tr '\n' ', ' | sed 's/, $//')
    if [[ -n "$fails" ]]; then
      echo "failing: ${fails}"
    else
      echo "test failures detected"
    fi
  fi
}

detail_release_build() {
  local output_file="$1" rc="$2"
  _detail_warn=""
  if [[ "$rc" -eq 0 ]]; then
    echo "binary built successfully"
  else
    local errs
    errs=$(grep -c '^error' "$output_file" 2>/dev/null) || errs=0
    echo "build failed (${errs} error(s))"
  fi
}

detail_duplication() {
  local output_file="$1" rc="$2"
  _detail_warn=""
  local max_exact="${TW_DUPES_MAX_EXACT_PERCENT:-5.0}"
  local max_near="${TW_DUPES_MAX_NEAR_PERCENT:-10.0}"

  if [[ "$rc" -eq 0 ]]; then
    local stats_file="${TW_DUPES_STATS_PATH:-target/duplication/dupes-stats.json}"
    if [[ -f "$stats_file" ]] && command -v jq >/dev/null 2>&1; then
      local exact near
      exact=$(jq -r '.exact_duplicate_percent // "?"' "$stats_file" 2>/dev/null || echo "?")
      near=$(jq -r '.near_duplicate_percent // "?"' "$stats_file" 2>/dev/null || echo "?")
      exact=$(printf '%.1f' "$exact" 2>/dev/null || echo "$exact")
      near=$(printf '%.1f' "$near" 2>/dev/null || echo "$near")
      if awk -v e="$exact" -v me="$max_exact" -v n="$near" -v mn="$max_near" \
         'BEGIN { exit (e > me || n > mn) ? 0 : 1 }' 2>/dev/null; then
        _detail_warn="WARN"
      fi
      echo "exact=${exact}% (max ${max_exact}%), near=${near}% (max ${max_near}%)"
    else
      local summary
      summary=$(grep -iE '(exact|duplicate|percent|total)' "$output_file" | tail -1)
      echo "${summary:-advisory analysis complete}"
    fi
  else
    echo "duplication thresholds exceeded"
  fi
}

detail_complexity_audit() {
  local output_file="$1" rc="$2"
  _detail_warn=""
  local files funcs crit_n med_n
  files=$(grep -oE 'analyzed: [0-9]+ files' "$output_file" | grep -oE '[0-9]+' || echo "?")
  funcs=$(grep -oE '[0-9]+ function blocks' "$output_file" | grep -oE '[0-9]+' || echo "?")
  crit_n=$(grep 'critical violations:' "$output_file" | head -1 \
    | grep -oE '[0-9]+' | awk '{s+=$1} END {print s+0}')
  med_n=$(grep 'medium violations:' "$output_file" | head -1 \
    | grep -oE '[0-9]+' | awk '{s+=$1} END {print s+0}')
  if [[ "$crit_n" -gt 0 || "$med_n" -gt 0 ]]; then
    _detail_warn="WARN"
  fi
  echo "${files} files, ${funcs} fns — ${crit_n} critical, ${med_n} medium"
}

detail_coverage() {
  local output_file="$1" rc="$2"
  _detail_warn=""

  local thresholds_file="${TW_COVERAGE_THRESHOLDS_FILE:-${SCRIPT_DIR}/coverage-thresholds.env}"
  local min_line=85 min_function=85 min_branch=75
  if [[ -f "$thresholds_file" ]]; then
    # shellcheck disable=SC1090
    source "$thresholds_file"
    min_line="${TW_COVERAGE_MIN_LINE:-85}"
    min_function="${TW_COVERAGE_MIN_FUNCTION:-85}"
    min_branch="${TW_COVERAGE_MIN_BRANCH:-75}"
  fi

  local line_cov fn_cov br_cov
  line_cov=$(grep -oE 'line coverage: [0-9.]+%' "$output_file" | head -1 | grep -oE '[0-9.]+')
  fn_cov=$(grep -oE 'function coverage: [0-9.]+%' "$output_file" | head -1 | grep -oE '[0-9.]+')
  br_cov=$(grep -oE 'branch coverage: [0-9.]+%' "$output_file" | head -1 | grep -oE '[0-9.]+')
  if [[ -n "$line_cov" ]]; then
    if awk -v l="$line_cov" -v ml="$min_line" \
       -v f="${fn_cov:-0}" -v mf="$min_function" \
       -v b="${br_cov:-0}" -v mb="$min_branch" \
       'BEGIN { exit (l < ml || f < mf || b < mb) ? 0 : 1 }' 2>/dev/null; then
      _detail_warn="WARN"
    fi
    echo "line=${line_cov}% (≥${min_line}%), fn=${fn_cov:-?}% (≥${min_function}%), branch=${br_cov:-?}% (≥${min_branch}%)"
  elif [[ "$rc" -eq 0 ]]; then
    echo "coverage collected"
  else
    # 1) threshold violations (the most actionable info)
    local threshold_failures
    threshold_failures=$(grep -E 'coverage .* is below threshold' "$output_file" \
      | sed 's/^error: //; s/^warning: //' \
      | head -3 \
      | tr '\n' '; ' \
      | sed 's/; $//')
    if [[ -n "$threshold_failures" ]]; then
      echo "$threshold_failures"
      return
    fi
    # 2) collection itself failed — show a brief reason
    local first_err
    first_err=$(grep -m1 '^error' "$output_file" 2>/dev/null \
      | sed 's/^error: //' \
      | sed 's|/[^ ]*/\.rustup/toolchains/[^ ]*/bin/||g' \
      | cut -c1-100)
    if [[ -n "$first_err" ]]; then
      echo "collection failed: ${first_err}"
    else
      echo "coverage failed (re-run with --verbose for details)"
    fi
  fi
}

# ── tool availability checks ─────────────────────────────────
has_rust_code_analysis() {
  command -v rust-code-analysis-cli >/dev/null 2>&1
}

has_cargo_dupes() {
  cargo dupes --help >/dev/null 2>&1
}

has_cargo_llvm_cov() {
  cargo llvm-cov --version >/dev/null 2>&1
}

has_jq() {
  command -v jq >/dev/null 2>&1
}

# ── step runner ───────────────────────────────────────────────
step_output_dir="$(mktemp -d)"
trap 'rm -rf "$step_output_dir"' EXIT

step_counter=0

run_step() {
  local classification="$1"
  local label="$2"
  local description="$3"
  local detail_fn="$4"
  shift 4

  ((step_counter++)) || true
  local output_file="${step_output_dir}/${step_counter}.log"

  # ── step header ──
  echo
  printf "  [%d] %-24s %s\n" "$step_counter" "$label" "$description"
  printf "      %-24s " ""

  # ── run command ──
  local rc=0
  if [[ "$VERBOSE" -eq 1 ]]; then
    echo
    "$@" > >(tee "$output_file") 2>&1 || rc=$?
  else
    "$@" > "$output_file" 2>&1 || rc=$?
  fi

  # ── extract detail ──
  _detail_warn=""
  local detail
  detail=$("$detail_fn" "$output_file" "$rc")

  local result icon
  if [[ "$rc" -ne 0 ]]; then
    result="FAIL"
    icon="✗"
  elif [[ "$_detail_warn" == "WARN" ]]; then
    result="WARN"
    icon="⚠"
  else
    result="PASS"
    icon="✓"
  fi

  if [[ "$VERBOSE" -eq 1 ]]; then
    printf "\n      %-24s " ""
  fi
  printf "%s %s\n" "$icon" "$detail"

  record "$label" "$classification" "$result" "$detail"
}

skip_step() {
  local label="$1"
  local description="$2"
  local reason="$3"

  ((step_counter++)) || true

  echo
  printf "  [%d] %-24s %s\n" "$step_counter" "$label" "$description"
  printf "      %-24s ⊘ SKIPPED — %s\n" "" "$reason"

  record "$label" "RECOMMENDED" "SKIP" "$reason"
}

# ══════════════════════════════════════════════════════════════
#  ENVIRONMENT CHECK
# ══════════════════════════════════════════════════════════════

echo "══════════════════════════════════════════════════════════════════"
echo " ENVIRONMENT"
echo "══════════════════════════════════════════════════════════════════"
echo

# required tools (always present for a Rust project)
printf "  %-28s " "cargo"
if command -v cargo >/dev/null 2>&1; then
  printf "✓  %s\n" "$(cargo --version 2>/dev/null)"
else
  printf "✗  NOT FOUND\n"
fi

printf "  %-28s " "rustfmt"
if cargo fmt --version >/dev/null 2>&1; then
  printf "✓  %s\n" "$(cargo fmt --version 2>/dev/null)"
else
  printf "✗  NOT FOUND\n"
fi

printf "  %-28s " "clippy"
if cargo clippy --version >/dev/null 2>&1; then
  printf "✓  %s\n" "$(cargo clippy --version 2>/dev/null)"
else
  printf "✗  NOT FOUND\n"
fi

# optional tools
printf "  %-28s " "jq"
if has_jq; then
  printf "✓  %s\n" "$(jq --version 2>/dev/null)"
else
  printf "⊘  not installed\n"
fi

printf "  %-28s " "rust-code-analysis-cli"
if has_rust_code_analysis; then
  printf "✓  installed\n"
else
  printf "⊘  not installed (cargo install --locked rust-code-analysis-cli --version 0.0.25)\n"
fi

printf "  %-28s " "cargo-dupes"
if has_cargo_dupes; then
  printf "✓  installed\n"
else
  printf "⊘  not installed (cargo install --locked cargo-dupes --version 0.2.1)\n"
fi

printf "  %-28s " "cargo-llvm-cov"
if has_cargo_llvm_cov; then
  printf "✓  %s\n" "$(cargo llvm-cov --version 2>/dev/null)"
else
  printf "⊘  not installed (cargo install --locked cargo-llvm-cov)\n"
fi

# summarise upcoming skips
declare -a skip_warnings=()
if ! has_rust_code_analysis || ! has_jq; then
  skip_warnings+=("Complexity gate (REQUIRED) — will SKIP")
fi
if ! has_cargo_dupes; then
  skip_warnings+=("Duplication analysis — will SKIP")
fi
if ! has_rust_code_analysis || ! has_jq; then
  skip_warnings+=("Complexity audit — will SKIP")
fi
if ! has_cargo_llvm_cov; then
  skip_warnings+=("Coverage — will SKIP")
fi

if [[ ${#skip_warnings[@]} -gt 0 ]]; then
  echo
  echo "  ⚠  The following checks will be skipped due to missing tools:"
  for w in "${skip_warnings[@]}"; do
    echo "     · ${w}"
  done
fi

# ══════════════════════════════════════════════════════════════
#  REQUIRED CHECKS
# ══════════════════════════════════════════════════════════════

echo
echo "══════════════════════════════════════════════════════════════════"
echo " REQUIRED"
echo "══════════════════════════════════════════════════════════════════"

run_step REQUIRED "Formatting" \
  "cargo fmt --all -- --check" \
  detail_formatting \
  cargo fmt --all -- --check

run_step REQUIRED "Clippy" \
  "cargo clippy -D warnings" \
  detail_clippy \
  cargo clippy --all-targets --all-features -- -D warnings

run_step REQUIRED "Clippy pedantic" \
  "pedantic + duplication-pattern lints" \
  detail_clippy \
  cargo clippy --all-targets --all-features -- \
    -D warnings \
    -D clippy::pedantic \
    -D clippy::if_same_then_else \
    -D clippy::match_same_arms \
    -D clippy::branches_sharing_code

if has_rust_code_analysis && has_jq; then
  run_step REQUIRED "Complexity gate" \
    "cyclomatic < 20, cognitive < 30, MI >= 30" \
    detail_complexity_gate \
    "$SCRIPT_DIR/complexity-gate.sh"
else
  missing=""
  has_rust_code_analysis || missing="rust-code-analysis-cli"
  has_jq || missing="${missing:+$missing, }jq"
  skip_step "Complexity gate" \
    "cyclomatic < 20, cognitive < 30, MI >= 30" \
    "missing: ${missing}"
  # record as REQUIRED skip (override the default RECOMMENDED from skip_step)
  step_kinds[-1]="REQUIRED"
fi

run_step REQUIRED "Type check" \
  "cargo check --all-targets --all-features" \
  detail_cargo_check \
  cargo check --all-targets --all-features

run_step REQUIRED "Tests" \
  "cargo test --all --all-features" \
  detail_tests \
  cargo test --all --all-features

run_step REQUIRED "Release build" \
  "cargo build --release" \
  detail_release_build \
  cargo build --release

# ══════════════════════════════════════════════════════════════
#  RECOMMENDED CHECKS
# ══════════════════════════════════════════════════════════════

echo
echo "══════════════════════════════════════════════════════════════════"
echo " RECOMMENDED"
echo "══════════════════════════════════════════════════════════════════"

if has_cargo_dupes; then
  run_step RECOMMENDED "Duplication analysis" \
    "exact/near duplicate detection" \
    detail_duplication \
    "$SCRIPT_DIR/duplication-gate.sh"
else
  skip_step "Duplication analysis" \
    "exact/near duplicate detection" \
    "cargo-dupes not installed"
fi

if has_rust_code_analysis && has_jq; then
  run_step RECOMMENDED "Complexity audit" \
    "tiered file/function complexity" \
    detail_complexity_audit \
    env TW_COMPLEXITY_FAIL_ON_CRITICAL=0 TW_COMPLEXITY_FAIL_ON_MEDIUM=0 \
      "$SCRIPT_DIR/complexity-audit.sh"
else
  missing=""
  has_rust_code_analysis || missing="rust-code-analysis-cli"
  has_jq || missing="${missing:+$missing, }jq"
  skip_step "Complexity audit" \
    "tiered file/function complexity" \
    "missing: ${missing}"
fi

if has_cargo_llvm_cov; then
  run_step RECOMMENDED "Coverage" \
    "line/function/branch thresholds" \
    detail_coverage \
    env TW_COVERAGE_FAIL_ON_THRESHOLD=0 \
      "$SCRIPT_DIR/coverage.sh"
else
  skip_step "Coverage" \
    "line/function/branch thresholds" \
    "cargo-llvm-cov not installed"
fi

# ══════════════════════════════════════════════════════════════
#  REPORT
# ══════════════════════════════════════════════════════════════

print_report() {
  local total=${#step_names[@]}
  local req_pass=0 req_fail=0 req_skip=0
  local rec_pass=0 rec_fail=0 rec_warn=0 rec_skip=0

  for i in $(seq 0 $((total - 1))); do
    local kind="${step_kinds[$i]}"
    local result="${step_results[$i]}"
    if [[ "$kind" == "REQUIRED" ]]; then
      case "$result" in
        PASS) ((req_pass++)) ;;
        FAIL) ((req_fail++)) ;;
        SKIP) ((req_skip++)) ;;
      esac
    else
      case "$result" in
        PASS) ((rec_pass++)) ;;
        WARN) ((rec_warn++)) ;;
        FAIL) ((rec_fail++)) ;;
        SKIP) ((rec_skip++)) ;;
      esac
    fi
  done

  local req_total=$((req_pass + req_fail + req_skip))
  local rec_run=$((rec_pass + rec_warn + rec_fail))
  local rec_total=$((rec_run + rec_skip))
  local skip_total=$((req_skip + rec_skip))

  echo
  echo "══════════════════════════════════════════════════════════════════"
  echo " CHECK REPORT"
  echo "══════════════════════════════════════════════════════════════════"
  echo
  echo " REQUIRED"
  echo " ──────────────────────────────────────────────────────────────"

  for i in $(seq 0 $((total - 1))); do
    [[ "${step_kinds[$i]}" != "REQUIRED" ]] && continue
    local icon
    case "${step_results[$i]}" in
      PASS) icon="✓" ;;
      FAIL) icon="✗" ;;
      SKIP) icon="⊘" ;;
    esac
    local detail="${step_details[$i]}"
    if [[ -n "$detail" ]]; then
      printf "  %s  %-24s %s\n" "$icon" "${step_names[$i]}" "$detail"
    else
      printf "  %s  %s\n" "$icon" "${step_names[$i]}"
    fi
  done

  echo
  echo " RECOMMENDED"
  echo " ──────────────────────────────────────────────────────────────"

  for i in $(seq 0 $((total - 1))); do
    [[ "${step_kinds[$i]}" != "RECOMMENDED" ]] && continue
    [[ "${step_results[$i]}" == "SKIP" ]] && continue
    local icon
    case "${step_results[$i]}" in
      PASS) icon="✓" ;;
      WARN) icon="⚠" ;;
      FAIL) icon="✗" ;;
    esac
    local detail="${step_details[$i]}"
    if [[ -n "$detail" ]]; then
      printf "  %s  %-24s %s\n" "$icon" "${step_names[$i]}" "$detail"
    else
      printf "  %s  %s\n" "$icon" "${step_names[$i]}"
    fi
  done

  if [[ "$skip_total" -gt 0 ]]; then
    echo
    echo " NOT RUN"
    echo " ──────────────────────────────────────────────────────────────"
    for i in $(seq 0 $((total - 1))); do
      [[ "${step_results[$i]}" != "SKIP" ]] && continue
      printf "  ⊘  %-24s %s\n" "${step_names[$i]}" "${step_details[$i]}"
    done
  fi

  echo
  echo "══════════════════════════════════════════════════════════════════"

  local verdict
  if [[ "$req_fail" -eq 0 && "$req_skip" -eq 0 ]]; then
    verdict="PASS"
  elif [[ "$req_fail" -gt 0 ]]; then
    verdict="FAIL"
  else
    verdict="PASS (with skips)"
  fi

  local summary="${req_pass}/${req_total} required passed"
  if [[ "$rec_total" -gt 0 ]]; then
    summary="${summary}, ${rec_pass}/${rec_run} recommended passed"
  fi
  if [[ "$rec_warn" -gt 0 ]]; then
    summary="${summary}, ${rec_warn} warning(s)"
  fi
  if [[ "$skip_total" -gt 0 ]]; then
    summary="${summary}, ${skip_total} skipped"
  fi

  printf " RESULT: %s  (%s)\n" "$verdict" "$summary"
  echo "══════════════════════════════════════════════════════════════════"

  if [[ "$VERBOSE" -eq 0 ]]; then
    echo
    echo " Tip: run with --verbose for full tool output"
  fi

  if [[ "$req_fail" -gt 0 ]]; then
    return 1
  fi
  return 0
}

print_report
