# server docs

В этой папке хранится документация модуля `apps/server`.

## Документы

## Mandatory core modules baseline

Для `apps/server` обязательные core-модули платформы:
- `rustok-index` — **Core, critical**
- `rustok-tenant` — **Core, critical**
- `rustok-rbac` — **Core, critical**
- `rustok-core` — **Core, critical**
- `rustok-outbox` — **Core, critical**
- `rustok-telemetry` — **Core, critical**

Агентам: считать этот список обязательным baseline платформы. Логика tenant-toggle/disable для core-модулей централизованно контролируется в `ModuleLifecycleService`.

- [`library-stack.md`](./library-stack.md) — основные backend-библиотеки сервера и их роль (framework, HTTP, ORM, GraphQL, runtime, observability).
- [`health.md`](./health.md) — health/readiness probes и текущие dependency checks сервера.
- [`event-transport.md`](./event-transport.md) — как работает конфигурация и runtime-пайплайн транспорта событий.
- [`event-flow-contract.md`](../../../docs/architecture/event-flow-contract.md) — канонический контракт полного event-пути (publish → outbox → delivery → consumer/read-model).
- [`loco/README.md`](./loco/README.md) — Loco-specific контекст, workflow для агентов и freshness-политика upstream snapshot.
- [`LOCO_FEATURE_SUPPORT.md`](./LOCO_FEATURE_SUPPORT.md) — decision matrix по Loco-функционалу vs самопису (anti-duplication baseline), включая статус Mailer/Workers/Storage и текущее состояние кэширования.
- [`loco-core-integration-plan.md`](./loco-core-integration-plan.md) — текущий status doc по server/Loco integration, live capabilities и residual scope.
- [`CORE_VERIFICATION_PLAN.md`](./CORE_VERIFICATION_PLAN.md) — периодический чеклист верификации ядра (13 секций, grep-проверки, таблица антипаттернов).
- Framework deviation checklist: для каждого нового отклонения от framework/runtime baseline обязателен checklist из [`docs/standards/forbidden-actions.md`](../../../docs/standards/forbidden-actions.md#64-framework-deviation-checklist-обязателен-для-каждого-нового-отклонения) (benchmark evidence, failure-mode table, rollback strategy, owner sign-off).
- [`upstream-libraries/README.md`](./upstream-libraries/README.md) — локальный snapshot актуальной внешней документации по ключевым crate сервера.
- Cleanup/maintenance: background cleanup task supports `sessions`, `rbac-report`, `rbac-backfill`, `rbac-backfill-rollback` targets (`cargo loco task --name cleanup --args "target=<target>"`); RBAC backfill supports safety controls `dry_run=true`, `limit=<N>`, `continue_on_error=true`, `exclude_user_ids=<uuid,...>`, `exclude_roles=<role,...>` and optional rollback snapshot output `rollback_file=<path>`; rollback target consumes `source=<rollback_file>` and supports `dry_run=true`; rollback now removes only role assignments captured in snapshot (role-targeted revert); app `truncate` hook now performs ordered deletion of server foundation tables (`release`, `build`, `tenant_modules`, `sessions`, `users`, `tenants`).
- RBAC rollout migration helpers удалены из live repo после финального перехода на single-engine `casbin_only`; детали cutover при необходимости фиксируются только в ADR/architecture docs и не являются частью текущего authorization path.
- Rich-text migration job (blog/forum legacy markdown → `rt_json_v1`): `cargo run -p rustok-server --bin migrate_legacy_richtext -- --tenant-id=<uuid> [--dry-run] [--checkpoint-file=...]`; job выполняет конвертацию + server-side sanitize/validation + safe update с retry, выводит метрики `processed/succeeded/failed/skipped`, поддерживает идемпотентный restart через checkpoint и предназначен для tenant-by-tenant rollout (с backup-based rollback на tenant scope).
- Auth/password reset: GraphQL `forgot_password` now dispatches reset emails via SMTP (`rustok.email` settings, credentials optional for local relay) with safe no-send fallback when email delivery is disabled.
- Auth/password reset: REST `POST /api/auth/reset/confirm` теперь отзывает все активные сессии пользователя (через `revoked_at`), что выравнивает policy с GraphQL reset-путём.
- Auth/user lifecycle: GraphQL `create_user` теперь в одной транзакции создаёт пользователя и назначает RBAC связи (`user_roles`/permissions) через `RbacService::assign_role_permissions`.
- Auth/RBAC resolver: `RbacService` использует модульный `rustok-rbac::RuntimePermissionResolver`; в server остаются adapter-реализации `RelationPermissionStore`/`PermissionCache`/`RoleAssignmentStore` (SeaORM + Moka + service wiring), а публичные write-path операции (`assign_role_permissions`/`replace_user_role`) также идут через модульный resolver API.
- Auth/RBAC resolver: `RbacService::get_user_permissions` использует in-memory cache (TTL 60s) с structured cache hit/miss логами и инвалидацией при изменении relation-ролей пользователя.
- Auth/RBAC consistency: REST RBAC extractors (`extractors/rbac.rs`) используют общую wildcard-семантику `resource:manage` через `rustok_rbac::has_effective_permission_in_set`, без локального дублирования permission-логики.
- Auth/RBAC observability: `/metrics` публикует `rustok_rbac_permission_cache_hits`, `rustok_rbac_permission_cache_misses`, `rustok_rbac_permission_checks_allowed`, `rustok_rbac_permission_checks_denied`, `rustok_rbac_claim_role_mismatch_total`, `rustok_rbac_engine_decisions_casbin_total`, `rustok_rbac_engine_eval_duration_ms_total`, `rustok_rbac_engine_eval_duration_samples`, а также consistency-gauges `rustok_rbac_users_without_roles_total`, `rustok_rbac_orphan_user_roles_total`, `rustok_rbac_orphan_role_permissions_total`, `rustok_rbac_consistency_query_failures_total`, `rustok_rbac_consistency_query_latency_ms_total`, `rustok_rbac_consistency_query_latency_samples`.
- Dev onboarding: `seed_development` creates/updates an idempotent demo tenant (`demo`), demo users, and enables core modules for local environments.
- Admin UI embedding: embedded `/admin` assets are compiled only with Cargo feature `embed-admin-assets` (disabled by default for CI/check environments without frontend artifacts). When enabled, build `apps/admin/dist` before compiling `apps/server`; when disabled, `/admin/*` returns `503 Service Unavailable` with an explicit message that embedding is disabled.
- Build pipeline: `BuildService::request_build` now publishes `BuildRequested` via configurable `BuildEventPublisher`; `EventBusBuildEventPublisher` maps it to `DomainEvent::BuildRequested`, while default noop publisher logs skipped dispatch when no runtime wiring is provided.
- Event consumer runtime: long-lived server consumers (`server_event_forwarder`, GraphQL build-progress subscription) follow a shared contract from `rustok-core`: `Lagged -> warn + metric`, `Closed -> explicit stop`, reindex decision path documented in `docs/architecture/events.md`.
- Tenant cache invalidation: Redis pubsub listener now uses supervised resubscribe with fixed backoff instead of one-shot startup; operational signals go through `rustok_event_consumer_restarted_total` and incident handling stays in the central events runbook.
- Health/readiness: `tenant_cache_invalidation` is now exposed in `/health/ready`, and current listener state is exported as `rustok_tenant_invalidation_listener_status`; see [`health.md`](./health.md).
- Auth/session lifecycle: GraphQL `sign_out`, `change_password`, `reset_password` теперь используют soft-revoke через `sessions.revoked_at` (вместо hard delete) и выровнены по поведению с REST (`sign_out` отзывает только текущую сессию, `change_password` — все остальные, `reset_password` — все активные).

- Auth/lifecycle extraction: REST handlers и GraphQL mutations для `register/sign_in`, `login/sign_in`, `refresh`, `change_password`, `reset_password`, `update_profile` теперь маршрутизируют бизнес-логику через общий `AuthLifecycleService` (transport adapters остаются тонкими).

- Auth/observability: `/metrics` публикует auth lifecycle counters `auth_password_reset_sessions_revoked_total`, `auth_change_password_sessions_revoked_total`, `auth_flow_inconsistency_total`, `auth_login_inactive_user_attempt_total`; первые два счётчика отражают количество реально отозванных сессий (rows affected), а счётчики ведутся в `AuthLifecycleService` для rollout-периода remediation-плана.

- Auth/error contracts: `AuthLifecycleService` использует типизированные ошибки (`AuthLifecycleError`), а REST/GraphQL делают единообразный transport-specific mapping без дублирования строковых веток.

- Auth rollout controls: канонические release gates, stop-the-line условия и rollback-процедура ведутся централизованно в `docs/architecture/api.md` (раздел «Auth lifecycle consistency и release-gate»); remediation backlog закрыт, в релизах используется operational handoff через `scripts/auth_release_gate.sh --require-all-gates`.
- Auth rollout controls: helper `scripts/auth_release_gate.sh` автоматизирует сбор локального integration evidence (`cargo test -p rustok-server auth_lifecycle` + `cargo test -p rustok-server auth`), всегда формирует markdown gate-report с полями для parity/security evidence и завершает прогон с non-zero exit code при падении любого локального integration auth-среза.
- RBAC/seed consistency: `seed_user` теперь вызывает `RbacService::assign_role_permissions` после создания пользователя, гарантируя наличие `user_roles` для всех seed-пользователей (dev bootstrap).


