#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
UPSTREAM_REPO="${LOCO_UPSTREAM_REPO:-https://github.com/loco-rs/loco.git}"
UPSTREAM_REF="${LOCO_UPSTREAM_REF:-main}"
TARGET_DIR="$ROOT_DIR/apps/server/docs/loco/upstream"
VERSION_FILE="$TARGET_DIR/VERSION"
TMP_DIR="$(mktemp -d)"

cleanup() {
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

echo "[sync-loco-docs] repo: $UPSTREAM_REPO"
echo "[sync-loco-docs] ref:  $UPSTREAM_REF"

git clone --depth 1 --branch "$UPSTREAM_REF" "$UPSTREAM_REPO" "$TMP_DIR/loco"
RESOLVED_COMMIT="$(git -C "$TMP_DIR/loco" rev-parse HEAD)"

DOCS_SOURCE=""
for candidate in \
  "$TMP_DIR/loco/docs" \
  "$TMP_DIR/loco/site/content/docs" \
  "$TMP_DIR/loco/website/content/docs"; do
  if [[ -d "$candidate" ]]; then
    DOCS_SOURCE="$candidate"
    break
  fi
done

if [[ -z "$DOCS_SOURCE" ]]; then
  echo "[sync-loco-docs] ERROR: Could not locate docs directory in upstream repo." >&2
  exit 1
fi

STAGE_DIR="$TMP_DIR/stage"
mkdir -p "$STAGE_DIR"

# Keep only framework-relevant guides for this repository.
# Keywords map to routing/controllers/models/migrations/auth/config/testing/background jobs, etc.
INCLUDE_REGEX='(routing|route|controller|model|migration|auth|config|configuration|test|testing|background|job|queue|worker|middleware|request|response)'

while IFS= read -r -d '' file; do
  rel="${file#"$DOCS_SOURCE/"}"
  lower_rel="$(echo "$rel" | tr '[:upper:]' '[:lower:]')"

  if [[ "$lower_rel" =~ $INCLUDE_REGEX ]] || [[ "$rel" == "README.md" ]] || [[ "$rel" == "SUMMARY.md" ]]; then
    mkdir -p "$STAGE_DIR/$(dirname "$rel")"
    cp "$file" "$STAGE_DIR/$rel"
  fi
done < <(find "$DOCS_SOURCE" -type f \( -name '*.md' -o -name '*.mdx' \) -print0)

# Preserve canonical snapshot README and replace synced docs payload.
find "$TARGET_DIR" -mindepth 1 -maxdepth 1 ! -name 'README.md' ! -name 'VERSION' -exec rm -rf {} +

if compgen -G "$STAGE_DIR/*" > /dev/null; then
  cp -R "$STAGE_DIR"/* "$TARGET_DIR"/
else
  echo "[sync-loco-docs] WARNING: no matching docs found with current filters." >&2
fi

cat > "$VERSION_FILE" <<META
source_repo=${UPSTREAM_REPO}
source_ref=${UPSTREAM_REF}
source_commit=${RESOLVED_COMMIT}
snapshot_date_utc=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
filters_regex=${INCLUDE_REGEX}
META

echo "[sync-loco-docs] done"
echo "[sync-loco-docs] docs source: $DOCS_SOURCE"
echo "[sync-loco-docs] commit: $RESOLVED_COMMIT"
