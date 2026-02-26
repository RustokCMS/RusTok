#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/rbac_relation_staging.sh"

fail() {
  echo "[FAIL] $*" >&2
  exit 1
}

pass() {
  echo "[PASS] $*"
}

make_mock_cargo() {
  local dir="$1"
  cat > "$dir/mock-cargo" <<'MOCK'
#!/usr/bin/env bash
set -euo pipefail

echo "mock cargo $*"
args="$*"

# emit report JSON when cleanup rbac-report output=<file> is requested
if [[ "$args" == *"target=rbac-report"* ]]; then
  output_file="$(printf '%s' "$args" | sed -n 's/.*output=\([^ ]*\).*/\1/p')"
  if [[ -n "$output_file" ]]; then
    mkdir -p "$(dirname "$output_file")"
    if [[ -n "${MOCK_REPORT_PROFILE:-}" && "$output_file" == *"post_apply"* ]]; then
      case "$MOCK_REPORT_PROFILE" in
        regression)
          cat > "$output_file" <<'JSON'
{"users_without_roles_total":5,"orphan_user_roles_total":1,"orphan_role_permissions_total":1}
JSON
          ;;
        improved)
          cat > "$output_file" <<'JSON'
{"users_without_roles_total":0,"orphan_user_roles_total":0,"orphan_role_permissions_total":0}
JSON
          ;;
        *)
          ;;
      esac
    elif [[ -n "${MOCK_REPORT_PROFILE:-}" && "$output_file" == *"post_rollback"* ]]; then
      cat > "$output_file" <<'JSON'
{"users_without_roles_total":1,"orphan_user_roles_total":0,"orphan_role_permissions_total":0}
JSON
    else
      cat > "$output_file" <<'JSON'
{"users_without_roles_total":1,"orphan_user_roles_total":0,"orphan_role_permissions_total":0}
JSON
    fi
  fi
fi

if [[ -n "${MOCK_TOUCH_ROLLBACK_FILE:-}" && "$args" == *"target=rbac-backfill"* && "$args" != *"dry_run=true"* ]]; then
  file="$(printf '%s' "$args" | sed -n 's/.*rollback_file=\([^ ]*\).*/\1/p')"
  if [[ -n "$file" ]]; then
    mkdir -p "$(dirname "$file")"
    echo "[]" > "$file"
  fi
fi
MOCK
  chmod +x "$dir/mock-cargo"
}

test_missing_rollback_source_fails() {
  local tmp
  tmp="$(mktemp -d)"
  make_mock_cargo "$tmp"

  set +e
  RUSTOK_CARGO_BIN="$tmp/mock-cargo" "$SCRIPT" --run-rollback-dry --artifacts-dir "$tmp/artifacts" >"$tmp/out.log" 2>&1
  local code=$?
  set -e

  [[ $code -eq 1 ]] || fail "expected exit 1 when rollback source is missing"
  rg -q "Rollback source file is required" "$tmp/out.log" || fail "expected missing rollback source message"
  pass "rollback dry-run without snapshot fails fast"
}

test_rollback_source_allows_dry_run() {
  local tmp
  tmp="$(mktemp -d)"
  make_mock_cargo "$tmp"
  echo '[]' > "$tmp/existing.rollback.json"

  RUSTOK_CARGO_BIN="$tmp/mock-cargo" "$SCRIPT" --run-rollback-dry --rollback-source "$tmp/existing.rollback.json" --artifacts-dir "$tmp/artifacts" >"$tmp/out.log" 2>&1

  rg -q "target=rbac-backfill-rollback source=$tmp/existing.rollback.json dry_run=true" "$tmp/out.log" || fail "expected rollback dry-run to use provided source"
  rg -q "Done. Report:" "$tmp/out.log" || fail "expected report generation"
  pass "rollback dry-run uses provided snapshot source"
}

test_apply_creates_snapshot_and_rollback_apply_uses_it() {
  local tmp
  tmp="$(mktemp -d)"
  make_mock_cargo "$tmp"

  MOCK_TOUCH_ROLLBACK_FILE=1 RUSTOK_CARGO_BIN="$tmp/mock-cargo" "$SCRIPT" --run-apply --run-rollback-apply --artifacts-dir "$tmp/artifacts" >"$tmp/out.log" 2>&1

  rg -q "target=rbac-backfill-rollback source=$tmp/artifacts/rbac_backfill_" "$tmp/out.log" || fail "expected rollback apply to use generated snapshot"
  rg -q "continue_on_error=false" "$tmp/out.log" || fail "expected rollback apply args to include continue_on_error"
  pass "apply+rollback apply path uses generated snapshot"
}

test_fail_on_regression_blocks_run() {
  local tmp
  tmp="$(mktemp -d)"
  make_mock_cargo "$tmp"

  set +e
  MOCK_TOUCH_ROLLBACK_FILE=1 MOCK_REPORT_PROFILE=regression RUSTOK_CARGO_BIN="$tmp/mock-cargo" "$SCRIPT" --run-apply --fail-on-regression --artifacts-dir "$tmp/artifacts" >"$tmp/out.log" 2>&1
  local code=$?
  set -e

  [[ $code -eq 1 ]] || fail "expected --fail-on-regression to fail when post-apply regresses"
  rg -q "Invariant regression detected" "$tmp/out.log" || fail "expected regression warning"
  pass "fail-on-regression blocks invariant regressions"
}

test_report_contains_invariant_diff_section() {
  local tmp
  tmp="$(mktemp -d)"
  make_mock_cargo "$tmp"

  MOCK_TOUCH_ROLLBACK_FILE=1 MOCK_REPORT_PROFILE=improved RUSTOK_CARGO_BIN="$tmp/mock-cargo" "$SCRIPT" --run-apply --artifacts-dir "$tmp/artifacts" >"$tmp/out.log" 2>&1

  local report
  report="$(rg -o 'Done\. Report: .*' "$tmp/out.log" | sed 's/Done\. Report: //')"
  [[ -n "$report" ]] || fail "expected report path in output"
  [[ -f "$report" ]] || fail "expected report file to exist"

  rg -q "Invariant diff: pre-check vs post-apply" "$report" || fail "expected invariant diff section"
  rg -q "delta -1" "$report" || fail "expected improved delta in report"
  pass "report includes invariant diff summary"
}

main() {
  test_missing_rollback_source_fails
  test_rollback_source_allows_dry_run
  test_apply_creates_snapshot_and_rollback_apply_uses_it
  test_fail_on_regression_blocks_run
  test_report_contains_invariant_diff_section
  echo "All tests passed."
}

main
