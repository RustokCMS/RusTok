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
- [`event-transport.md`](./event-transport.md) — как работает конфигурация и runtime-пайплайн транспорта событий.
- [`event-flow-contract.md`](../../../docs/architecture/event-flow-contract.md) — канонический контракт полного event-пути (publish → outbox → delivery → consumer/read-model).
- [`loco/README.md`](./loco/README.md) — Loco-specific контекст, workflow для агентов и freshness-политика upstream snapshot.
- [`LOCO_FEATURE_SUPPORT.md`](./LOCO_FEATURE_SUPPORT.md) — decision matrix по Loco-функционалу vs самопису (anti-duplication baseline), включая статус Mailer/Workers/Storage и текущее состояние кэширования.
- [`upstream-libraries/README.md`](./upstream-libraries/README.md) — локальный snapshot актуальной внешней документации по ключевым crate сервера.
- Cleanup/maintenance: background cleanup task supports `sessions`, `rbac-report`, `rbac-backfill`, `rbac-backfill-rollback` targets (`cargo loco task --name cleanup --args "target=<target>"`); RBAC backfill supports safety controls `dry_run=true`, `limit=<N>`, `continue_on_error=true`, `exclude_user_ids=<uuid,...>`, `exclude_roles=<role,...>` and optional rollback snapshot output `rollback_file=<path>`; rollback target consumes `source=<rollback_file>` and supports `dry_run=true`; app `truncate` hook now performs ordered deletion of server foundation tables (`release`, `build`, `tenant_modules`, `sessions`, `users`, `tenants`).
- Auth/password reset: GraphQL `forgot_password` now dispatches reset emails via SMTP (`rustok.email` settings, credentials optional for local relay) with safe no-send fallback when email delivery is disabled.
- Auth/password reset: REST `POST /api/auth/reset/confirm` теперь отзывает все активные сессии пользователя (через `revoked_at`), что выравнивает policy с GraphQL reset-путём.
- Auth/user lifecycle: GraphQL `create_user` теперь в одной транзакции создаёт пользователя и назначает RBAC связи (`user_roles`/permissions) через `AuthService::assign_role_permissions`.
- Auth/RBAC resolver: `AuthService` использует модульный `rustok-rbac::RuntimePermissionResolver`; в server остаются adapter-реализации `RelationPermissionStore`/`PermissionCache`/`RoleAssignmentStore` (SeaORM + Moka + service wiring), а публичные write-path операции (`assign_role_permissions`/`replace_user_role`) также идут через модульный resolver API.
- Auth/RBAC resolver: `AuthService::get_user_permissions` использует in-memory cache (TTL 60s) с structured cache hit/miss логами и инвалидацией при изменении relation-ролей пользователя.
- Auth/RBAC consistency: REST RBAC extractors (`extractors/rbac.rs`) используют общую wildcard-семантику `resource:manage` через `rustok_rbac::has_effective_permission_in_set`, без локального дублирования permission-логики.
- Auth/RBAC observability: `/metrics` публикует `rustok_rbac_permission_cache_hits`, `rustok_rbac_permission_cache_misses`, `rustok_rbac_permission_checks_allowed`, `rustok_rbac_permission_checks_denied`, `rustok_rbac_claim_role_mismatch_total`, `rustok_rbac_decision_mismatch_total`, `rustok_rbac_shadow_compare_failures_total`, а также consistency-gauges `rustok_rbac_users_without_roles_total`, `rustok_rbac_orphan_user_roles_total`, `rustok_rbac_orphan_role_permissions_total`, `rustok_rbac_consistency_query_failures_total`, `rustok_rbac_consistency_query_latency_ms_total`, `rustok_rbac_consistency_query_latency_samples`.
- Auth/RBAC dual-read rollout: permission checks in `AuthService::{has_permission,has_any_permission,has_all_permissions}` can run shadow legacy-role comparison behind `RUSTOK_RBAC_AUTHZ_MODE=dual_read`; relation decision stays authoritative, mismatches increment `rustok_rbac_decision_mismatch_total` and are logged as `rbac_decision_mismatch`; dual-read also uses a short-lived in-memory cache for legacy role lookups to avoid extra DB round-trips per check, and shadow compare failures are logged without affecting allow/deny outcome.
- Dev onboarding: `seed_development` creates/updates an idempotent demo tenant (`demo`), demo users, and enables core modules for local environments.
- Build pipeline: `BuildService::request_build` now publishes `BuildRequested` via configurable `BuildEventPublisher`; `EventBusBuildEventPublisher` maps it to `DomainEvent::BuildRequested`, while default noop publisher logs skipped dispatch when no runtime wiring is provided.
- Auth/session lifecycle: GraphQL `sign_out`, `change_password`, `reset_password` теперь используют soft-revoke через `sessions.revoked_at` (вместо hard delete) и выровнены по поведению с REST (`sign_out` отзывает только текущую сессию, `change_password` — все остальные, `reset_password` — все активные).

- Auth/lifecycle extraction: REST handlers и GraphQL mutations для `register/sign_in`, `login/sign_in`, `refresh`, `change_password`, `reset_password` теперь маршрутизируют бизнес-логику через общий `AuthLifecycleService` (transport adapters остаются тонкими).

- Auth/error contracts: `AuthLifecycleService` использует типизированные ошибки (`AuthLifecycleError`), а REST/GraphQL делают единообразный transport-specific mapping без дублирования строковых веток.

- Auth rollout controls: канонические release gates, stop-the-line условия и rollback-процедура ведутся централизованно в `docs/architecture/user-auth-consistency-remediation-plan.md` (раздел 8), этот README хранит только краткий changelog.
- RBAC/seed consistency: `seed_user` теперь вызывает `AuthService::assign_role_permissions` после создания пользователя, гарантируя наличие `user_roles` для всех seed-пользователей (dev bootstrap).
