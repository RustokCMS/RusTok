# Loco.rs upstream documentation snapshot

> ⚠️ This directory stores a **local snapshot/copy of official Loco.rs documentation** and is the canonical source for framework behavior in this repository.

## Snapshot metadata

- Snapshot date: **2026-02-11 (UTC)**
- Snapshot version: **bootstrap placeholder (run sync script to pin real upstream revision)**
- Upstream repository: <https://github.com/loco-rs/loco>
- Upstream commit/tag: **pending initial sync** (written to `VERSION` by `scripts/docs/sync_loco_docs.sh`)

## Why this exists

- Give contributors and AI agents a stable, reviewable documentation baseline.
- Reduce hallucinations by pinning guidance to a known upstream revision.
- Keep local project notes separate from framework source-of-truth.

## Refresh process

Use:

```bash
scripts/docs/sync_loco_docs.sh
```

After sync, check `VERSION` and review changed files before commit.
