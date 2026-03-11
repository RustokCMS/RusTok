#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 ]]; then
  echo "Usage: $0 --tenant-id=<uuid> [--dry-run] [--batch-size=N] [--max-retries=N] [--retry-delay-ms=N] [--checkpoint-file=path]" >&2
  exit 1
fi

cargo run -p rustok-server --bin migrate_legacy_richtext -- "$@"
