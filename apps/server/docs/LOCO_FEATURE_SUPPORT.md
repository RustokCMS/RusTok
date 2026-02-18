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
**Решение:** не дублировать это параллельной Loco queue-runtime реализацией; оставить собственную очередь/воркеры ради расширяемости и архитектурной консистентности.

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

## 6) Быстрый roadmap по замечаниям ревью

1. **Mailer migration:** перевести password reset delivery на Loco Mailer API.
2. **Storage unification:** внедрить Loco storage abstraction как обязательный слой для модульных upload/use-cases.
3. **Queue consistency:** задокументировать (ADR/архдок) окончательное правило «queue/workers только самопис» и не дублировать Loco job queue.
4. **Caching clarity:** при следующих изменениях tenancy cache — обновлять этот документ и `apps/server/docs/README.md` одновременно.

---

## 7) Sources

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
