#!/usr/bin/env python3
"""Checks and updates Loco upstream docs snapshot metadata."""

from __future__ import annotations

import argparse
from datetime import date, datetime, timezone
from pathlib import Path
import sys

DEFAULT_VERSION_PATH = Path("apps/server/docs/loco/upstream/VERSION")
WARNING_DAYS = 30
FAIL_DAYS = 60


def parse_snapshot_date(version_path: Path) -> date:
    if not version_path.exists():
        raise FileNotFoundError(f"Missing required file: {version_path}")

    snapshot_value: str | None = None
    for raw_line in version_path.read_text(encoding="utf-8").splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue

        if "=" in line:
            key, value = line.split("=", 1)
            if key.strip() in {"snapshot_date", "date", "metadata_date"}:
                snapshot_value = value.strip()
                break

    if snapshot_value is None:
        raise ValueError(
            "VERSION must contain a 'snapshot_date=YYYY-MM-DD' entry "
            "(also supports date=... or metadata_date=...)."
        )

    try:
        return datetime.strptime(snapshot_value, "%Y-%m-%d").date()
    except ValueError as exc:
        raise ValueError(
            f"Invalid snapshot date '{snapshot_value}' in {version_path}. "
            "Expected format: YYYY-MM-DD"
        ) from exc


def snapshot_age_days(snapshot_date: date, today: date | None = None) -> int:
    current = today or datetime.now(timezone.utc).date()
    return (current - snapshot_date).days


def write_version_file(version_path: Path) -> None:
    today = datetime.now(timezone.utc).date().isoformat()
    version_path.parent.mkdir(parents=True, exist_ok=True)
    version_path.write_text(
        "\n".join(
            [
                "# Loco upstream snapshot metadata",
                "# Updated by: make docs-sync-loco",
                f"snapshot_date={today}",
            ]
        )
        + "\n",
        encoding="utf-8",
    )


def cmd_check(version_path: Path) -> int:
    snapshot_date = parse_snapshot_date(version_path)
    age_days = snapshot_age_days(snapshot_date)

    print(f"Snapshot date: {snapshot_date.isoformat()}")
    print(f"Snapshot age: {age_days} day(s)")

    if age_days > FAIL_DAYS:
        print(
            f"::error::Loco upstream snapshot is stale ({age_days} days old). "
            f"Maximum allowed age: {FAIL_DAYS} days. Run `make docs-sync-loco`.",
            file=sys.stderr,
        )
        return 1

    if age_days > WARNING_DAYS:
        print(
            f"::warning::Loco upstream snapshot is getting old ({age_days} days). "
            f"Please refresh soon with `make docs-sync-loco`."
        )

    return 0


def cmd_sync(version_path: Path) -> int:
    write_version_file(version_path)
    print(f"Updated {version_path} with today's snapshot date.")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "command",
        choices=["check", "sync"],
        help="check snapshot staleness or sync timestamp",
    )
    parser.add_argument(
        "--version-file",
        type=Path,
        default=DEFAULT_VERSION_PATH,
        help=f"Path to VERSION metadata file (default: {DEFAULT_VERSION_PATH})",
    )
    return parser


def main() -> int:
    parser = build_parser()
    args = parser.parse_args()

    if args.command == "check":
        return cmd_check(args.version_file)
    return cmd_sync(args.version_file)


if __name__ == "__main__":
    raise SystemExit(main())
