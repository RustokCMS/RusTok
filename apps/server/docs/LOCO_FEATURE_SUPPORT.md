# RusToK Server — Loco.rs Feature Support & Anti-Duplication Matrix

**Date:** 2026-02-18  
**Loco.rs Version:** `0.16` (workspace dependency)  
**Purpose:** сохранить полный обзор реализованного server-функционала (включая auth и доменные API), при этом явно зафиксировать границы: где используем Loco, где сознательно используем самопис.

---

## 1) Полная матрица: Loco capability vs реализация RusToK

| Capability area | Loco support | Реализовано сейчас | Source of truth (целевое) | Риск дублей | Решение |
|---|---|---|---|---|---|
| Application hooks (`Hooks`) | ✅ | `boot`, `routes`, `after_routes`, `truncate`, `register_tasks`, `initializers`, `connect_workers`, `seed` | **Loco hooks** | Низкий | Оставить на Loco |
| Конфигурация приложения | ✅ | `development.yaml`/`test.yaml`, `auth.jwt`, custom `settings.rustok.*` | **Loco config + typed project settings** | Низкий | Оставить как есть |
| REST/GraphQL роутинг | ✅ | `AppRoutes` + Axum layers, GraphQL endpoint | **Loco + project controllers** | Низкий | Оставить как есть |
| ORM/migrations/entities | ✅ (SeaORM stack) | migration crate + entities + модели | **Loco/SeaORM stack** | Низкий | Оставить как есть |
| Auth framework primitives | ✅ (patterns/hooks) | JWT, refresh sessions, password reset tokens, RBAC domain wiring | **Project domain logic atop Loco runtime** | Средний | Не дублировать infra-слой Loco, но доменную auth-логику оставить своей |
| Tasks (`cargo loco task`) | ✅ | `CleanupTask` зарегистрирован | **Loco Tasks** | Низкий | Оставить на Loco |
| Initializers | ✅ | `TelemetryInitializer` через Loco API | **Loco Initializers** | Низкий | Оставить на Loco |
| Mailer subsystem | ✅ | Сейчас кастомный SMTP service (`lettre`) + GraphQL forgot_password | **Loco Mailer** | **Высокий** | Мигрировать почтовый flow на Loco Mailer API |
| Workers/queue subsystem | ✅ | Сейчас собственный event-driven outbox relay worker | **RusToK custom (осознанно)** | Средний | Очереди/воркеры оставить самописными (не дублировать Loco queue runtime) |
| Storage abstraction (uploads/assets) | ✅ | Единый Loco storage для всех модулей пока не внедрён | **Loco Storage** | **Высокий** | Ввести общий storage adapter/policy через Loco для всех модулей |
| Кэширование tenancy | N/A (project concern) | custom tenant cache + negative cache + invalidation + metrics | **RusToK custom** | Низкий | Оставить самопис (platform-specific) |
| Event bus / outbox transport | N/A (project architecture) | memory/outbox/iggy transport + relay worker | **RusToK custom** | Низкий | Оставить самопис |

## Governance register

Реестр ниже — обязательная входная точка для архитектурных решений по Loco-capabilities в `apps/server`.

| Capability | Runtime owner (current) | Source of truth (target) | ADR / reference (required) | Decision status | Next review date | Кодовые точки |
|---|---|---|---|---|---|---|
| Application hooks (`Hooks`) | `apps/server` + `loco_rs` runtime | Loco hooks contract + `apps/server/src/app.rs` as integration layer | `apps/server/docs/loco/README.md`; `DECISIONS/2026-02-19-core-server-module-bundles-routing.md` | Accepted | 2026-06-01 | `apps/server/src/app.rs` |
| Конфигурация приложения (`Config` + `settings.rustok.*`) | `apps/server` | Loco config (`config/*.yaml`) + typed settings in server | `apps/server/docs/loco/README.md`; `docs/architecture/overview.md` | Accepted | 2026-06-01 | `apps/server/src/common/settings.rs`; `apps/server/config/development.yaml`; `apps/server/config/test.yaml` |
| REST/GraphQL routing | `apps/server` | Loco `AppRoutes` + server controllers/graphql modules | `DECISIONS/2026-02-19-core-server-module-bundles-routing.md`; `docs/architecture/api.md` | Accepted | 2026-06-01 | `apps/server/src/app.rs`; `apps/server/src/controllers/mod.rs`; `apps/server/src/graphql/mod.rs` |
| ORM/migrations/entities | `apps/server` migration + SeaORM entities | SeaORM stack in server app + migration crate | `docs/architecture/database.md`; `apps/server/docs/README.md` | Accepted | 2026-06-01 | `apps/server/migration/src/lib.rs`; `apps/server/src/models/mod.rs` |
| Auth framework primitives (JWT/sessions/reset/RBAC wiring) | `apps/server` + `rustok-core` + `rustok-rbac` | Domain auth logic поверх Loco runtime | `DECISIONS/2026-02-26-auth-lifecycle-unification-session-invalidation.md`; `DECISIONS/2026-03-05-rbac-relation-only-final-cutover-gate.md` | Accepted | 2026-05-20 | `apps/server/src/services/auth.rs`; `apps/server/src/graphql/auth/mutation.rs`; `apps/server/src/controllers/auth.rs` |
| Tasks (`cargo loco task`) | `apps/server` via Loco task runtime | Loco tasks API with server task registration | `apps/server/docs/README.md`; `docs/guides/quickstart.md` | Accepted | 2026-06-01 | `apps/server/src/tasks/mod.rs`; `apps/server/src/tasks/cleanup.rs`; `apps/server/src/app.rs` |
| Initializers | `apps/server` via Loco initializer runtime | Loco initializer API + project initializers | `apps/server/docs/README.md`; `docs/guides/observability-quickstart.md` | Accepted | 2026-06-01 | `apps/server/src/initializers/mod.rs`; `apps/server/src/initializers/telemetry.rs`; `apps/server/src/app.rs` |
| Mailer subsystem | `apps/server` (custom `EmailService` on `lettre`) | **Loco Mailer API** (+ provider config in `settings`) | `apps/server/docs/loco/README.md`; `docs/architecture/api.md` | Proposed | 2026-04-15 | `apps/server/src/services/email.rs`; `apps/server/src/graphql/auth/mutation.rs`; `apps/server/src/common/settings.rs` |
| Workers/queue subsystem | `apps/server` + `rustok-outbox` | RusToK event-driven worker runtime (без Loco queue duplication) | `DECISIONS/2026-03-11-queue-runtime-source-of-truth-outbox.md`; `docs/architecture/event-flow-contract.md`; `docs/standards/transactional-outbox.md` | Accepted | 2026-05-01 | `apps/server/src/app.rs`; `apps/server/src/services/event_transport_factory.rs`; `crates/rustok-outbox/src/relay.rs` |
| Storage abstraction (uploads/assets) | `apps/server` (частичные ad-hoc use-cases) | **Loco Storage** shared policy/adapters for modules | `apps/server/docs/loco/README.md`; `docs/architecture/modules.md` | Needs review | 2026-04-15 | `apps/server/src/app.rs`; `apps/server/src/controllers/content.rs`; `apps/server/src/controllers/pages.rs` |
| Tenancy caching | `apps/server` + `rustok-core` cache backends | RusToK custom tenancy cache (`tenant.rs`) + shared cache backend contract | `docs/architecture/tenancy.md`; `docs/guides/observability-quickstart.md` | Accepted | 2026-05-01 | `apps/server/src/middleware/tenant.rs`; `apps/server/src/middleware/tenant_cache_v3.rs`; `crates/rustok-core/src/cache.rs` |
| Event bus / transport (`memory|outbox|iggy`) | `apps/server` + `rustok-events` + `rustok-outbox` | RusToK event transport contract + transactional outbox flow | `DECISIONS/2026-02-19-rustok-events-canonical-contract.md`; `docs/architecture/events.md`; `apps/server/docs/event-transport.md` | Accepted | 2026-05-01 | `apps/server/src/services/event_transport_factory.rs`; `apps/server/src/services/build_request_events.rs`; `apps/server/src/workers/outbox_relay.rs` |

---

## 2) Что реализовано в сервере (полный функциональный срез)

### 2.1 Core Loco lifecycle & app bootstrap

Реализовано в `impl Hooks for App`:
- `app_name`, `app_version`;
- `boot` на `create_app::<Self, Migrator>`;
- `routes` с регистрацией health/metrics/auth/graphql и domain controllers;
- `after_routes` с tenant middleware + runtime extensions;
- `truncate` (не stub, а реальная очистка таблиц в dependency order);
- `register_tasks`;
- `initializers`;
- `connect_workers`;
- `seed`.

### 2.2 Configuration system

- Environment yaml-конфиги (`development.yaml`, `test.yaml`).
- Loco `auth.jwt` конфигурация.
- Typed settings-расширение через `settings.rustok.*` (`tenant`, `search`, `features`, `rate_limit`, `events`, `email`).

### 2.3 Controllers & API surface

- REST controllers: health, metrics, auth, swagger, pages.
- Domain controllers: commerce, content, blog, forum.
- GraphQL endpoint + domain GraphQL modules (`auth`, `commerce`, `content`, `blog`, `forum`, loaders, persisted queries).

### 2.4 Models / ORM / persistence

- SeaORM integration активна.
- Migration crate подключён.
- Основные сущности и модели используются в auth/tenancy/domain flows.

### 2.5 Authentication & authorization (важно: не удалено)

Реализовано и используется:
- JWT access token + refresh token flow.
- Session management в БД (`sessions`).
- Password hashing (`argon2`) и verify.
- Password reset flow (forgot/reset mutations, reset token encoding/decoding, revoke sessions after reset).
- RBAC permissions/roles assignment через `AuthService` + `rustok-rbac`/domain entities.

### 2.6 Middleware / tenancy / rate-limit context

- Tenant resolution middleware (header/domain modes).
- Validation tenant identifiers.
- Cache + negative cache для tenant resolution.
- Middleware layering через `after_routes`.
- Rate-limit настройки есть в `settings`; реальное поведение завязано на серверные middleware/services.

### 2.7 Background processing / events

- Outbox relay worker запускается из `connect_workers`.
- Event runtime создаётся из конфигурации транспорта (`memory` / `outbox` / `iggy`).
- Event-driven подход остаётся приоритетным для очередей и интеграций.

### 2.8 Tasks & Initializers

- `cleanup` task зарегистрирован, поддерживает `sessions`, `cache`, full cleanup.
- `TelemetryInitializer` подключён через Loco initializer API.

### 2.9 Testing support

- Loco testing feature включён в server dev-dependencies.
- Набор unit/integration тестов в серверном модуле присутствует (см. `apps/server/tests` и inline tests в модулях).

---

## 3) Что в Loco есть, но у нас должно/решено быть иначе

### 3.1 Mailer (должен быть через Loco)

**Сейчас:** password reset email отправляется кастомным `EmailService` (`lettre`).  
**Целевое решение:** использовать Loco Mailer как основной integration contract, сохранив проектные provider-настройки и observability.

### 3.2 Workers/Queue (осознанно самопис)

**Сейчас:** outbox relay worker + event-driven pipeline.  
**Решение:** не дублировать это параллельной Loco queue-runtime реализацией; оставить собственную очередь/воркеры ради расширяемости и архитектурной консистентности. Policy anchor: `DECISIONS/2026-03-11-queue-runtime-source-of-truth-outbox.md`.

### 3.3 Storage abstraction (должно быть едино через Loco)

**Сейчас:** единого Loco storage abstraction для всех модулей нет.  
**Целевое решение:** ввести общий Loco storage слой (policy + adapters), чтобы модульные upload/storage use-cases не расползались на ad-hoc реализации.

---

## 4) Кэширование: текущее состояние (детально)

### 4.1 Tenant cache (основной путь)

`middleware/tenant.rs` реализует:
- versioned cache keys,
- positive cache + negative cache,
- anti-stampede request coalescing (`in_flight` + `Notify`),
- Redis pub/sub invalidation channel (`tenant.cache.invalidate`) при включённом `redis-cache`,
- метрики (`hits/misses/negative/coalesced`).

### 4.2 Cache backends (shared infra)

`rustok-core` предоставляет:
- `InMemoryCacheBackend` (Moka),
- `RedisCacheBackend` (feature-gated), включая circuit breaker.

В сервере используется общий CacheBackend-контракт с выбором backend по feature/runtime.

### 4.3 Cache observability

`/metrics` отдаёт tenant cache метрики `rustok_tenant_cache_*` (hits, misses, entries, negative indicators).

### 4.4 Tenant cache v3

`tenant_cache_v3.rs` присутствует как альтернативная реализация с circuit breaker + Moka моделью, но основной production path сейчас проходит через инфраструктуру `tenant.rs`.

---

## 5) Практические anti-duplication правила

1. Перед добавлением infra-функционала проверять, есть ли его зрелая реализация в Loco.
2. Для осознанных отклонений фиксировать rationale (как для queue/workers) в этом документе.
3. Не держать параллельные production-реализации одного слоя (Mailer/Storage/Queue) без миграционного плана.
4. Любое изменение в кэше должно сопровождаться требованиями к invalidation + метрикам.
5. Для новых модулей: использовать зафиксированный source of truth из матрицы раздела 1.

---

## 6) Loco Mailer + Storage roadmap (release phases)

Ниже фиксируется единый rollout-план для `Mailer` и `Storage`, чтобы не поддерживать параллельные production-потоки без явных gate-критериев.

| Phase | Цель | Mailer (code points + config keys) | Storage (code points + config keys) | Release gates / metrics |
|---|---|---|---|---|
| 0. Contract freeze | Зафиксировать integration contracts и schema ожидания до реализации адаптеров | Freeze API для password reset delivery в `apps/server/src/services/email.rs`; зафиксировать mapping `settings.rustok.email.*` (`enabled`, `from`, `reset_base_url`, `smtp.host`, `smtp.port`, `smtp.username`) в `apps/server/src/common/settings.rs` и `apps/server/config/development.yaml` | Freeze контракт `StorageAdapter` и policy-level SLA для upload/download; зафиксировать будущие ключи `settings.rustok.storage.*` и Loco `storage.*` как config source of truth | Gate: PR checklist подтверждает отсутствие breaking changes. Success: `delivery_error_rate` baseline собран; `p95_mailer_latency_ms` и `p95_storage_latency_ms` baseline собраны |
| 1. Adapter implementation | Реализовать Loco-based adapters без cutover трафика | Добавить Loco mailer adapter рядом с текущим SMTP sender (bridge в `apps/server/src/services/email.rs`) и feature flag `settings.rustok.email.provider=loco|smtp` | Ввести storage adapters (директория `apps/server/src/services/storage_adapters/`) и policy resolver из config (`settings.rustok.storage.provider`, `settings.rustok.storage.bucket`, `settings.rustok.storage.prefix`) | Gate: unit/integration tests для adapter parity. Success: functional parity == 100%, `fallback_rate < 5%` на тестовом трафике |
| 2. Dual-run / shadow | Запустить shadow-доставку и сравнить результаты | Основной send идёт через legacy SMTP path, Loco mailer выполняется shadow mode; расхождения логируются с correlation id | Основной storage path остаётся legacy, Loco adapter выполняет shadow upload/read-check без consumer impact | Gate: минимум 7 дней shadow без деградации. Success: `delivery_error_rate_delta <= 0.2pp`, `p95_latency_delta <= 10%`, `fallback_rate <= 2%` |
| 3. Cutover | Перевести production трафик на Loco adapters | Включить `settings.rustok.email.provider=loco` + rollback toggle в runtime config; наблюдение через `email_delivery_errors_total`, `email_send_latency_ms` | Включить `settings.rustok.storage.provider=loco` + rollback toggle; наблюдение через `storage_operation_errors_total`, `storage_operation_latency_ms` | Gate: SRE approval + on-call readiness. Success: 14 дней стабильности, `fallback_rate <= 1%`. Rollback trigger: 3 consecutive 5m windows с `delivery_error_rate > 2%` или `p95_latency_ms > 2x baseline` |
| 4. Legacy removal | Удалить legacy paths после стабилизации | Удалить legacy SMTP-only send path и флаги совместимости из `apps/server/src/services/email.rs`/`apps/server/src/common/settings.rs` | Удалить legacy storage branches в adapters/policy; оставить единый Loco storage runtime path | Gate: post-cutover retrospective + ADR update. Success: `fallback_rate = 0` 30 дней; rollback trigger деактивирован после cleanup window |

### Release gates (детализация по Mailer/Storage)

#### Mailer

- **Primary metrics:**
  - `delivery_error_rate` (доля неуспешных отправок password reset / transactional email),
  - `p95_mailer_latency_ms`,
  - `fallback_rate` (доля запросов, ушедших на legacy SMTP path).
- **Hard release gate для cutover:**
  - `delivery_error_rate <= 1.0%` в течение 24h,
  - `p95_mailer_latency_ms <= baseline * 1.25`,
  - `fallback_rate <= 1.0%`.
- **Rollback trigger:**
  - `delivery_error_rate > 2.0%` 3 окна подряд по 5 минут,
  - или `p95_mailer_latency_ms > baseline * 2.0`,
  - или spike 5xx от provider API > 3%.

#### Storage

- **Primary metrics:**
  - `storage_error_rate` (upload/download/list failures),
  - `p95_storage_latency_ms`,
  - `fallback_rate` (доля операций, ушедших в legacy storage path).
- **Hard release gate для cutover:**
  - `storage_error_rate <= 0.5%` в течение 24h,
  - `p95_storage_latency_ms <= baseline * 1.30`,
  - `fallback_rate <= 1.0%`.
- **Rollback trigger:**
  - `storage_error_rate > 1.0%` 3 окна подряд по 5 минут,
  - или `p95_storage_latency_ms > baseline * 2.0`,
  - или рост timeout-rate > 2%.

## 7) Operational runbook (incidents / rollback)

- Incident/rollback runbook для фаз 2–4: [`LOCO_FEATURE_SUPPORT.md#6-loco-mailer--storage-roadmap-release-phases`](./LOCO_FEATURE_SUPPORT.md#6-loco-mailer--storage-roadmap-release-phases).
- Обязательная процедура при тревогах по gate-метрикам:
  1. Зафиксировать инцидент с phase ID (`mailer-shadow`, `mailer-cutover`, `storage-cutover`).
  2. Включить rollback toggle (provider=`smtp` или legacy storage provider) в runtime config.
  3. Проверить восстановление SLA в 2 последовательных окнах наблюдения.
  4. Сохранить post-incident summary и обновить этот roadmap перед повторным rollout.

---

## 8) Sources

- `apps/server/src/app.rs`
- `apps/server/src/controllers/mod.rs`
- `apps/server/src/controllers/metrics.rs`
- `apps/server/src/graphql/mod.rs`
- `apps/server/src/graphql/auth/mutation.rs`
- `apps/server/src/services/email.rs`
- `apps/server/src/services/event_transport_factory.rs`
- `apps/server/src/tasks/mod.rs`
- `apps/server/src/tasks/cleanup.rs`
- `apps/server/src/initializers/mod.rs`
- `apps/server/src/initializers/telemetry.rs`
- `apps/server/src/middleware/tenant.rs`
- `apps/server/src/middleware/tenant_cache_v3.rs`
- `apps/server/src/common/settings.rs`
- `apps/server/config/development.yaml`
- `apps/server/config/test.yaml`
- `crates/rustok-core/src/cache.rs`
- `crates/rustok-core/src/context.rs`
- `apps/server/Cargo.toml`
- `Cargo.toml`
