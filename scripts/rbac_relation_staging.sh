#!/usr/bin/env bash
set -euo pipefail

# Staged RBAC relation migration helper for staging environments.
# Runs cleanup task targets in a safe sequence and stores logs/artifacts.

usage() {
  cat <<'USAGE'
Usage:
  scripts/rbac_relation_staging.sh [options]

Options:
  --env <name>                Loco environment (default: staging)
  --limit <N>                 Optional candidate limit for staged backfill
  --exclude-user-ids <list>   Comma-separated UUIDs to skip
  --exclude-roles <list>      Comma-separated legacy roles to skip
  --continue-on-error         Continue backfill/rollback on per-user failures
  --run-apply                 Run non-dry-run backfill step
  --run-rollback-dry          Run rollback dry-run after backfill
  --run-rollback-apply        Run actual rollback (dangerous; explicit)
  --rollback-source <file>    Use existing rollback snapshot file instead of generated one
  --fail-on-regression        Exit non-zero if post-check invariants regress vs pre-check
  --require-zero-post-apply   Exit non-zero if post-apply invariants are not all zero
  --require-zero-post-rollback Exit non-zero if post-rollback invariants are not all zero
  --require-report-artifacts  Exit non-zero if expected report JSON artifacts are missing
  --artifacts-dir <dir>       Output folder for logs/report (default: artifacts/rbac-staging)
  --help                      Show this message

Environment:
  RUSTOK_CARGO_BIN            Override cargo executable path (default: cargo)

Examples:
  scripts/rbac_relation_staging.sh --run-apply --run-rollback-dry
  scripts/rbac_relation_staging.sh --limit 100 --exclude-roles super_admin --run-apply
USAGE
}

ENV_NAME="staging"
LIMIT=""
EXCLUDE_USER_IDS=""
EXCLUDE_ROLES=""
CONTINUE_ON_ERROR="false"
RUN_APPLY="false"
RUN_ROLLBACK_DRY="false"
RUN_ROLLBACK_APPLY="false"
ROLLBACK_SOURCE=""
FAIL_ON_REGRESSION="false"
REQUIRE_ZERO_POST_APPLY="false"
REQUIRE_ZERO_POST_ROLLBACK="false"
REQUIRE_REPORT_ARTIFACTS="false"
ARTIFACTS_DIR="artifacts/rbac-staging"
CARGO_BIN="${RUSTOK_CARGO_BIN:-cargo}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --env)
      ENV_NAME="$2"; shift 2 ;;
    --limit)
      LIMIT="$2"; shift 2 ;;
    --exclude-user-ids)
      EXCLUDE_USER_IDS="$2"; shift 2 ;;
    --exclude-roles)
      EXCLUDE_ROLES="$2"; shift 2 ;;
    --continue-on-error)
      CONTINUE_ON_ERROR="true"; shift ;;
    --run-apply)
      RUN_APPLY="true"; shift ;;
    --run-rollback-dry)
      RUN_ROLLBACK_DRY="true"; shift ;;
    --run-rollback-apply)
      RUN_ROLLBACK_APPLY="true"; shift ;;
    --rollback-source)
      ROLLBACK_SOURCE="$2"; shift 2 ;;
    --fail-on-regression)
      FAIL_ON_REGRESSION="true"; shift ;;
    --require-zero-post-apply)
      REQUIRE_ZERO_POST_APPLY="true"; shift ;;
    --require-zero-post-rollback)
      REQUIRE_ZERO_POST_ROLLBACK="true"; shift ;;
    --require-report-artifacts)
      REQUIRE_REPORT_ARTIFACTS="true"; shift ;;
    --artifacts-dir)
      ARTIFACTS_DIR="$2"; shift 2 ;;
    --help)
      usage; exit 0 ;;
    *)
      echo "Unknown option: $1" >&2
      usage
      exit 1 ;;
  esac
done

if [[ "$REQUIRE_ZERO_POST_APPLY" == "true" && "$RUN_APPLY" != "true" ]]; then
  echo "--require-zero-post-apply requires --run-apply." >&2
  exit 1
fi

if [[ "$REQUIRE_ZERO_POST_ROLLBACK" == "true" && "$RUN_ROLLBACK_APPLY" != "true" ]]; then
  echo "--require-zero-post-rollback requires --run-rollback-apply." >&2
  exit 1
fi

mkdir -p "$ARTIFACTS_DIR"
TS="$(date -u +%Y%m%dT%H%M%SZ)"
GENERATED_ROLLBACK_FILE="$ARTIFACTS_DIR/rbac_backfill_${TS}.rollback.json"
ROLLBACK_FILE="$GENERATED_ROLLBACK_FILE"
if [[ -n "$ROLLBACK_SOURCE" ]]; then
  ROLLBACK_FILE="$ROLLBACK_SOURCE"
fi
REPORT_FILE="$ARTIFACTS_DIR/rbac_relation_stage_report_${TS}.md"
PRECHECK_JSON="$ARTIFACTS_DIR/rbac_report_pre_${TS}.json"
DRYRUN_JSON="$ARTIFACTS_DIR/rbac_backfill_dry_run_${TS}.json"
POST_APPLY_JSON="$ARTIFACTS_DIR/rbac_report_post_apply_${TS}.json"
POST_APPLY_BACKFILL_JSON="$ARTIFACTS_DIR/rbac_backfill_apply_${TS}.json"
ROLLBACK_DRYRUN_JSON="$ARTIFACTS_DIR/rbac_backfill_rollback_dry_run_${TS}.json"
ROLLBACK_APPLY_JSON="$ARTIFACTS_DIR/rbac_backfill_rollback_apply_${TS}.json"
POST_ROLLBACK_JSON="$ARTIFACTS_DIR/rbac_report_post_rollback_${TS}.json"

build_args() {
  local target="$1"
  local args="target=${target}"

  if [[ -n "$LIMIT" ]]; then
    args+=" limit=${LIMIT}"
  fi
  if [[ -n "$EXCLUDE_USER_IDS" ]]; then
    args+=" exclude_user_ids=${EXCLUDE_USER_IDS}"
  fi
  if [[ -n "$EXCLUDE_ROLES" ]]; then
    args+=" exclude_roles=${EXCLUDE_ROLES}"
  fi
  if [[ "$CONTINUE_ON_ERROR" == "true" ]]; then
    args+=" continue_on_error=true"
  fi

  echo "$args"
}

extract_metric_from_json() {
  local file="$1"
  local key="$2"
  python - "$file" "$key" <<'PY'
import json
import sys
path, key = sys.argv[1], sys.argv[2]
with open(path, 'r', encoding='utf-8') as fh:
    payload = json.load(fh)
value = payload.get(key)
if not isinstance(value, int):
    raise SystemExit(f"invalid or missing integer metric '{key}' in {path}")
print(value)
PY
}

append_backfill_summary() {
  local title="$1"
  local file="$2"

  if [[ ! -f "$file" ]]; then
    return 0
  fi

  python - "$title" "$file" >> "$REPORT_FILE" <<'PY'
import json
import sys

_, title, path = sys.argv
with open(path, 'r', encoding='utf-8') as fh:
    payload = json.load(fh)

if not isinstance(payload, dict):
    raise SystemExit(f"unexpected report format in {path}: expected JSON object")

priority_keys = [
    "dry_run",
    "candidates_total",
    "entries_total",
    "fixed_users",
    "reverted",
    "failed_users",
    "failed",
    "users_without_roles_total_before",
    "users_without_roles_total_after",
    "orphan_user_roles_total_before",
    "orphan_user_roles_total_after",
    "orphan_role_permissions_total_before",
    "orphan_role_permissions_total_after",
]

seen = set()
ordered = []
for key in priority_keys:
    if key in payload:
        ordered.append((key, payload[key]))
        seen.add(key)

for key in sorted(payload):
    if key in seen:
        continue
    ordered.append((key, payload[key]))

print()
print(f"## {title}")
print()
for key, value in ordered:
    print(f"- {key}: {value}")
PY
}

append_invariant_summary() {
  local title="$1"
  local before_file="$2"
  local after_file="$3"

  if [[ ! -f "$before_file" || ! -f "$after_file" ]]; then
    return 0
  fi

  local users_before users_after users_delta
  local orphan_roles_before orphan_roles_after orphan_roles_delta
  local orphan_permissions_before orphan_permissions_after orphan_permissions_delta

  users_before="$(extract_metric_from_json "$before_file" "users_without_roles_total")"
  users_after="$(extract_metric_from_json "$after_file" "users_without_roles_total")"
  users_delta="$((users_after - users_before))"

  orphan_roles_before="$(extract_metric_from_json "$before_file" "orphan_user_roles_total")"
  orphan_roles_after="$(extract_metric_from_json "$after_file" "orphan_user_roles_total")"
  orphan_roles_delta="$((orphan_roles_after - orphan_roles_before))"

  orphan_permissions_before="$(extract_metric_from_json "$before_file" "orphan_role_permissions_total")"
  orphan_permissions_after="$(extract_metric_from_json "$after_file" "orphan_role_permissions_total")"
  orphan_permissions_delta="$((orphan_permissions_after - orphan_permissions_before))"

  {
    echo
    echo "## ${title}"
    echo
    echo "- users_without_roles_total: ${users_before} -> ${users_after} (delta ${users_delta})"
    echo "- orphan_user_roles_total: ${orphan_roles_before} -> ${orphan_roles_after} (delta ${orphan_roles_delta})"
    echo "- orphan_role_permissions_total: ${orphan_permissions_before} -> ${orphan_permissions_after} (delta ${orphan_permissions_delta})"
  } >> "$REPORT_FILE"

  if [[ "$FAIL_ON_REGRESSION" == "true" ]]; then
    if (( users_delta > 0 || orphan_roles_delta > 0 || orphan_permissions_delta > 0 )); then
      echo "Invariant regression detected in ${title}. Check ${REPORT_FILE}." >&2
      exit 1
    fi
  fi
}

enforce_zero_invariants() {
  local file="$1"
  local stage="$2"

  if [[ ! -f "$file" ]]; then
    echo "Invariant zero-check failed for ${stage}: report file is missing (${file})." >&2
    echo "See report: ${REPORT_FILE}" >&2
    exit 1
  fi

  local users_without_roles
  local orphan_user_roles
  local orphan_role_permissions
  users_without_roles="$(extract_metric_from_json "$file" "users_without_roles_total")"
  orphan_user_roles="$(extract_metric_from_json "$file" "orphan_user_roles_total")"
  orphan_role_permissions="$(extract_metric_from_json "$file" "orphan_role_permissions_total")"

  if (( users_without_roles > 0 || orphan_user_roles > 0 || orphan_role_permissions > 0 )); then
    echo "Invariant zero-check failed for ${stage}." >&2
    echo "Expected all invariants to be 0, got: users_without_roles_total=${users_without_roles}, orphan_user_roles_total=${orphan_user_roles}, orphan_role_permissions_total=${orphan_role_permissions}." >&2
    echo "See report: ${REPORT_FILE}" >&2
    exit 1
  fi
}

require_rollback_source() {
  local reason="$1"
  if [[ ! -f "$ROLLBACK_FILE" ]]; then
    echo "Rollback source file is required for ${reason} but was not found: ${ROLLBACK_FILE}" >&2
    echo "Hint: run with --run-apply first, or pass --rollback-source <existing_snapshot.json>." >&2
    exit 1
  fi
}

require_report_artifact() {
  local file="$1"
  local stage="$2"

  if [[ "$REQUIRE_REPORT_ARTIFACTS" != "true" ]]; then
    return 0
  fi

  if [[ ! -f "$file" ]]; then
    echo "Missing required report artifact for ${stage}: ${file}" >&2
    echo "See report: ${REPORT_FILE}" >&2
    exit 1
  fi
}

run_step() {
  local name="$1"
  local args="$2"
  local log_file="$ARTIFACTS_DIR/${TS}_${name}.log"

  echo "==> ${name}: ${CARGO_BIN} loco task --name cleanup --env ${ENV_NAME} --args \"${args}\""
  "$CARGO_BIN" loco task --name cleanup --env "$ENV_NAME" --args "$args" | tee "$log_file"
}

# 1) Baseline
run_step "01_pre_report" "target=rbac-report output=${PRECHECK_JSON}"

# 2) Dry-run backfill
run_step "02_backfill_dry_run" "$(build_args rbac-backfill) dry_run=true rollback_file=${GENERATED_ROLLBACK_FILE} report_file=${DRYRUN_JSON}"
require_report_artifact "$DRYRUN_JSON" "backfill dry-run"

# 3) Apply backfill (optional)
if [[ "$RUN_APPLY" == "true" ]]; then
  run_step "03_backfill_apply" "$(build_args rbac-backfill) rollback_file=${GENERATED_ROLLBACK_FILE} report_file=${POST_APPLY_BACKFILL_JSON}"
  run_step "04_post_report" "target=rbac-report output=${POST_APPLY_JSON}"
  require_report_artifact "$POST_APPLY_BACKFILL_JSON" "backfill apply"
  require_report_artifact "$POST_APPLY_JSON" "post-apply consistency report"
  if [[ "$REQUIRE_ZERO_POST_APPLY" == "true" ]]; then
    enforce_zero_invariants "$POST_APPLY_JSON" "post-apply"
  fi
else
  echo "Skipping apply step (use --run-apply to enable)."
fi

# 4) Rollback dry-run (optional)
if [[ "$RUN_ROLLBACK_DRY" == "true" ]]; then
  require_rollback_source "rollback dry-run"
  run_step "05_rollback_dry_run" "target=rbac-backfill-rollback source=${ROLLBACK_FILE} dry_run=true report_file=${ROLLBACK_DRYRUN_JSON}"
  require_report_artifact "$ROLLBACK_DRYRUN_JSON" "rollback dry-run"
fi

# 5) Rollback apply (optional, explicit)
if [[ "$RUN_ROLLBACK_APPLY" == "true" ]]; then
  require_rollback_source "rollback apply"
  run_step "06_rollback_apply" "target=rbac-backfill-rollback source=${ROLLBACK_FILE} continue_on_error=${CONTINUE_ON_ERROR} report_file=${ROLLBACK_APPLY_JSON}"
  run_step "07_post_rollback_report" "target=rbac-report output=${POST_ROLLBACK_JSON}"
  require_report_artifact "$ROLLBACK_APPLY_JSON" "rollback apply"
  require_report_artifact "$POST_ROLLBACK_JSON" "post-rollback consistency report"
  if [[ "$REQUIRE_ZERO_POST_ROLLBACK" == "true" ]]; then
    enforce_zero_invariants "$POST_ROLLBACK_JSON" "post-rollback"
  fi
fi

cat > "$REPORT_FILE" <<REPORT
# RBAC relation staged migration report

- Timestamp (UTC): ${TS}
- Environment: ${ENV_NAME}
- Artifacts directory: ${ARTIFACTS_DIR}
- Generated rollback snapshot path: ${GENERATED_ROLLBACK_FILE}
- Pre-check JSON report: ${PRECHECK_JSON}
- Dry-run backfill JSON report: ${DRYRUN_JSON}
- Post-apply JSON report: ${POST_APPLY_JSON}
- Post-apply backfill JSON report: ${POST_APPLY_BACKFILL_JSON}
- Rollback dry-run JSON report: ${ROLLBACK_DRYRUN_JSON}
- Rollback apply JSON report: ${ROLLBACK_APPLY_JSON}
- Post-rollback JSON report: ${POST_ROLLBACK_JSON}
- Effective rollback source: ${ROLLBACK_FILE}
- Apply step enabled: ${RUN_APPLY}
- Rollback dry-run enabled: ${RUN_ROLLBACK_DRY}
- Rollback apply enabled: ${RUN_ROLLBACK_APPLY}
- Limit: ${LIMIT:-<none>}
- Excluded user IDs: ${EXCLUDE_USER_IDS:-<none>}
- Excluded roles: ${EXCLUDE_ROLES:-<none>}
- Continue on error: ${CONTINUE_ON_ERROR}
- Fail on regression: ${FAIL_ON_REGRESSION}
- Require zero invariants after apply: ${REQUIRE_ZERO_POST_APPLY}
- Require zero invariants after rollback: ${REQUIRE_ZERO_POST_ROLLBACK}
- Require report artifacts: ${REQUIRE_REPORT_ARTIFACTS}

## Generated logs

$(for f in "$ARTIFACTS_DIR"/${TS}_*.log; do
  [[ -e "$f" ]] || continue
  echo "- $(basename "$f")"
done)
REPORT

append_backfill_summary "Backfill dry-run summary" "$DRYRUN_JSON"
append_backfill_summary "Backfill apply summary" "$POST_APPLY_BACKFILL_JSON"
append_backfill_summary "Rollback dry-run summary" "$ROLLBACK_DRYRUN_JSON"
append_backfill_summary "Rollback apply summary" "$ROLLBACK_APPLY_JSON"
append_invariant_summary "Invariant diff: pre-check vs post-apply" "$PRECHECK_JSON" "$POST_APPLY_JSON"
append_invariant_summary "Invariant diff: pre-check vs post-rollback" "$PRECHECK_JSON" "$POST_ROLLBACK_JSON"

echo "Done. Report: ${REPORT_FILE}"
echo "Rollback snapshot: ${ROLLBACK_FILE}"
