#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORK_DIR="$(mktemp -d)"
trap 'rm -rf "$WORK_DIR"' EXIT

FAKE_CARGO="$WORK_DIR/fake-cargo.sh"
ARTIFACTS_DIR="$WORK_DIR/artifacts"

cat > "$FAKE_CARGO" <<'FAKE'
#!/usr/bin/env bash
set -euo pipefail

if [[ "$1" != "loco" || "$2" != "task" ]]; then
  echo "unexpected command" >&2
  exit 2
fi

ARGS=""
for ((i=1; i<=$#; i++)); do
  if [[ "${!i}" == "--args" ]]; then
    j=$((i+1))
    ARGS="${!j}"
    break
  fi
done

if [[ -z "$ARGS" ]]; then
  echo "missing --args" >&2
  exit 3
fi

target=""
output_file=""
report_file=""
for kv in $ARGS; do
  key="${kv%%=*}"
  val="${kv#*=}"
  case "$key" in
    target) target="$val" ;;
    output) output_file="$val" ;;
    report_file) report_file="$val" ;;
  esac
done

case "$target" in
  rbac-report)
    [[ -n "$output_file" ]] || exit 4
    cat > "$output_file" <<JSON
{"users_without_roles_total":0,"orphan_user_roles_total":0,"orphan_role_permissions_total":0}
JSON
    ;;
  rbac-backfill)
    [[ -n "$report_file" ]] || exit 5
    if [[ "$ARGS" == *"dry_run=true"* ]]; then
      cat > "$report_file" <<JSON
{"dry_run":true,"candidates_total":3,"fixed_users":0,"failed_users":0,"users_without_roles_total_before":3,"users_without_roles_total_after":3,"orphan_user_roles_total_before":0,"orphan_user_roles_total_after":0,"orphan_role_permissions_total_before":0,"orphan_role_permissions_total_after":0}
JSON
    else
      cat > "$report_file" <<JSON
{"dry_run":false,"candidates_total":3,"fixed_users":3,"failed_users":0,"users_without_roles_total_before":3,"users_without_roles_total_after":0,"orphan_user_roles_total_before":0,"orphan_user_roles_total_after":0,"orphan_role_permissions_total_before":0,"orphan_role_permissions_total_after":0}
JSON
      # create rollback snapshot if path provided
      for kv in $ARGS; do
        key="${kv%%=*}"; val="${kv#*=}"
        if [[ "$key" == "rollback_file" ]]; then
          echo '[]' > "$val"
        fi
      done
    fi
    ;;
  rbac-backfill-rollback)
    [[ -n "$report_file" ]] || exit 6
    if [[ "$ARGS" == *"dry_run=true"* ]]; then
      cat > "$report_file" <<JSON
{"dry_run":true,"entries_total":3,"reverted":0,"failed":0,"users_without_roles_total_before":0,"users_without_roles_total_after":0,"orphan_user_roles_total_before":0,"orphan_user_roles_total_after":0,"orphan_role_permissions_total_before":0,"orphan_role_permissions_total_after":0}
JSON
    else
      cat > "$report_file" <<JSON
{"dry_run":false,"entries_total":3,"reverted":3,"failed":0,"users_without_roles_total_before":0,"users_without_roles_total_after":0,"orphan_user_roles_total_before":0,"orphan_user_roles_total_after":0,"orphan_role_permissions_total_before":0,"orphan_role_permissions_total_after":0}
JSON
    fi
    ;;
  *)
    echo "unsupported target=$target" >&2
    exit 7
    ;;
esac
FAKE

chmod +x "$FAKE_CARGO"

RUSTOK_CARGO_BIN="$FAKE_CARGO" \
  "$ROOT_DIR/scripts/rbac_relation_staging.sh" \
  --run-apply \
  --run-rollback-dry \
  --run-rollback-apply \
  --require-report-artifacts \
  --require-zero-post-apply \
  --require-zero-post-rollback \
  --artifacts-dir "$ARTIFACTS_DIR" >/dev/null

REPORT_FILE="$(find "$ARTIFACTS_DIR" -maxdepth 1 -name 'rbac_relation_stage_report_*.md' | head -n 1)"
[[ -n "$REPORT_FILE" ]] || { echo "missing stage report" >&2; exit 8; }

rg -q "Rollback apply summary" "$REPORT_FILE"
rg -q "entries_total: 3" "$REPORT_FILE"
rg -q "Require report artifacts: true" "$REPORT_FILE"

echo "rbac_relation_staging smoke test passed"
