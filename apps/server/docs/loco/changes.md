# Loco.rs changes log (project-local)

Используйте этот файл как короткий журнал изменений server-паттернов,
чтобы разработчики и AI-агенты быстрее находили актуальные решения.

## Format

- YYYY-MM-DD — change summary
  - affected path(s)
  - migration/auth/routing impact
  - breaking/not-breaking

## Entries

- 2026-02-19 — Removed dead middleware files; fixed double event-runtime init in Hooks
  - `apps/server/src/middleware/mod.rs`
  - `apps/server/src/app.rs`
  - Removed: `tenant_v2.rs`, `tenant_cache_v2.rs`, `tenant_cache_v3.rs`, `block_rest_auth.rs` (unused, never wired into routes)
  - Fixed: `connect_workers` now reuses the `Arc<EventRuntime>` stored in `shared_store` by `after_routes` instead of calling `build_event_runtime` a second time (previously created a second transport instance that was not used by the event bus)
  - Not breaking

- 2026-02-11 — Switched Loco docs to upstream snapshot source-of-truth model
  - `apps/server/docs/loco/upstream/README.md`
  - `apps/server/docs/loco/upstream/VERSION`
  - `apps/server/docs/loco/README.md`
  - `scripts/docs/sync_loco_docs.sh`
  - Added canonical upstream snapshot workflow and agent guidance to consult upstream first
  - Not breaking

- 2026-02-11 — Added initial Loco context pack
  - `apps/server/docs/loco/README.md`
  - Added server-local guidance to reduce framework hallucinations
  - Not breaking
