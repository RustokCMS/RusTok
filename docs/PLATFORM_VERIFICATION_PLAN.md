# RusToK — Глобальный план верификации платформы

- **Дата создания:** 2026-02-26
- **Статус:** В процессе (часть фаз проверена в сессиях 2026-02-27)
- **Цель:** Из разрозненных модулей, написанных разными агентами, получить проверенную, работоспособную платформу с синхронизированной документацией

---

## Как пользоваться этим планом

Каждый раздел содержит конкретные проверки в формате чеклиста (`- [ ]`). Мы проходим по пунктам последовательно. При обнаружении проблемы — фиксим и ставим галочку. После прохождения всех секций платформа будет в верифицированном состоянии.

**Условные обозначения:**
- `[ ]` — не проверено
- `[x]` — проверено, ОК
- `[!]` — проверено, найдена проблема (описание ниже чекбокса)
- `[~]` — частично реализовано / требует доработки

**Принцип работы с ошибками:**
При обнаружении проблемы — **сначала исправляем** (код, конфиг, документация), затем ставим `[x]`.
Пункт `[!]` означает «найдено, но исправление заблокировано» (технический долг, требует ADR или отдельной задачи).
Не собираем коллекцию ошибок — исправляем по ходу.

**Расширяемость плана:**
Этот план — **living document**. В процессе верификации можно и нужно:
- Добавлять подпункты внутри существующих фаз, если обнаружены новые аспекты для проверки
- Детализировать проверки (разбивать один чекбокс на несколько)
- Добавлять заметки и ссылки на найденные проблемы прямо под чекбоксами
- Создавать новые секции `### X.N` внутри фазы

**Не нужно** создавать отдельный план для каждого нового набора проверок — расширяйте этот документ.

**Связь с зависимостями модулей:**
Полная схема зависимостей между модулями задокументирована в:
- [Граф зависимостей модулей (Mermaid)](./architecture/diagram.md) — 12 диаграмм, включая dependency graph
- [modules.toml](../modules.toml) — source of truth для `depends_on`
- [Категории модулей A/B/C](./architecture/modules.md) — compile-time vs runtime
- [Реестр компонентов](./modules/registry.md) — каталог всех crates и apps

---

## Оглавление

1. [Фаза 0: Компиляция и сборка](#фаза-0-компиляция-и-сборка)
2. [Фаза 1: Соответствие архитектуре](#фаза-1-соответствие-архитектуре)
3. [Фаза 2: Ядро платформы (Core Infrastructure)](#фаза-2-ядро-платформы)
4. [Фаза 3: Система авторизации и аутентификации](#фаза-3-авторизация-и-аутентификация)
5. [Фаза 4: RBAC](#фаза-4-rbac)
6. [Фаза 5: Multi-Tenancy](#фаза-5-multi-tenancy)
7. [Фаза 6: Событийная система](#фаза-6-событийная-система)
8. [Фаза 7: Доменные модули](#фаза-7-доменные-модули)
9. [Фаза 8: API — GraphQL](#фаза-8-api-graphql)
10. [Фаза 9: API — REST](#фаза-9-api-rest)
11. [Фаза 10: Фронтенды — Leptos](#фаза-10-фронтенды-leptos)
12. [Фаза 11: Фронтенды — Next.js](#фаза-11-фронтенды-nextjs)
13. [Фаза 12: Фронтенд-библиотеки](#фаза-12-фронтенд-библиотеки)
14. [Фаза 13: Интеграционные связи](#фаза-13-интеграционные-связи)
15. [Фаза 14: Тестовое покрытие](#фаза-14-тестовое-покрытие)
16. [Фаза 15: Observability и операционная готовность](#фаза-15-observability)
17. [Фаза 16: Синхронизация документации с кодом](#фаза-16-синхронизация-документации)
18. [Фаза 17: CI/CD и DevOps](#фаза-17-cicd)
19. [Фаза 18: Безопасность](#фаза-18-безопасность)
20. [Фаза 19: Антипаттерны и качество кода](#фаза-19-антипаттерны-и-качество-кода)
21. [Фаза 20: Правильность написания кода](#фаза-20-правильность-написания-кода-code-correctness)
22. [Фаза 21: Реестр найденных проблем](#фаза-21-реестр-найденных-проблем)
23. [Итоговый отчёт](#итоговый-отчёт)

---

## Фаза 0: Компиляция и сборка

### 0.1 Workspace-level сборка

- [ ] `cargo check --workspace` — весь workspace компилируется без ошибок
- [ ] `cargo check --workspace --all-features` — компиляция со всеми features
- [ ] `cargo clippy --workspace -- -D warnings` — нет warnings от clippy
- [ ] `cargo fmt --all -- --check` — код форматирован

### 0.2 Отдельные targets

- [ ] `cargo build -p rustok-server` — основной сервер собирается
- [ ] `cargo build -p rustok-admin` — Leptos admin собирается
- [ ] `cargo build -p rustok-storefront` — Leptos storefront собирается

### 0.6 Зависимости (Cargo)

- [x] `iggy` версия исправлена: `0.9.2` → `0.9.0` (crates.io не имел 0.9.2)
  - Исправлено в `Cargo.toml` (workspace) и `crates/rustok-iggy-connector/Cargo.toml`
- [ ] `cargo update` не приводит к конфликтам версий
- [ ] `Cargo.lock` зафиксирован и корректен

### 0.3 Каждый crate компилируется независимо

- [ ] `rustok-core`
- [ ] `rustok-events`
- [ ] `rustok-outbox`
- [ ] `rustok-tenant`
- [ ] `rustok-rbac`
- [ ] `rustok-index`
- [ ] `rustok-content`
- [ ] `rustok-commerce`
- [ ] `rustok-blog`
- [ ] `rustok-forum`
- [ ] `rustok-pages`
- [ ] `alloy-scripting`
- [ ] `rustok-telemetry`
- [ ] `rustok-iggy`
- [ ] `rustok-iggy-connector`
- [ ] `rustok-mcp`
- [ ] `rustok-test-utils`

### 0.4 Frontend builds

- [ ] `apps/next-admin`: `npm install && npm run build` проходит
- [ ] `apps/next-frontend`: `npm install && npm run build` проходит
- [ ] `UI/leptos`: cargo build собирается
- [ ] `UI/next`: TypeScript компилируется

### 0.5 Вспомогательные инструменты

- [ ] `cargo build -p xtask` — xtask собирается
- [ ] `make help` — Makefile содержит актуальные targets
- [ ] Docker: `docker-compose.yml` валиден (`docker compose config`)
- [ ] Docker: `docker-compose.full-dev.yml` валиден

---

## Фаза 1: Соответствие архитектуре

### 1.1 Module Registry vs modules.toml

**Файлы:** `apps/server/src/modules/mod.rs`, `modules.toml`

- [x] Все модули из `modules.toml` зарегистрированы в `build_registry()`
- [ ] `validate_registry_vs_manifest()` вызывается при старте сервера в `app.rs`
- [x] Slug'и в registry совпадают со slug'ами в `modules.toml`
- [x] `required = true` в `modules.toml` совпадает с `ModuleKind::Core` в коде
  - Исправлено: `content` убран из раздела required в `modules.toml` (был `required = true`, но `ContentModule::kind()` возвращает `ModuleKind::Optional` по умолчанию)
- [x] `depends_on` в `modules.toml` совпадают с `dependencies()` в `RusToKModule` impl

### 1.2 Cargo.toml workspace members

- [ ] Все директории из `crates/*` присутствуют в `Cargo.toml` workspace members
- [ ] `apps/server`, `apps/admin`, `apps/storefront` — в workspace members
- [ ] `UI/leptos` — в workspace members
- [ ] `benches`, `xtask` — в workspace members
- [ ] Нет orphan-директорий в `crates/` без Cargo.toml

### 1.3 Категоризация компонентов

**Проверяем соответствие Категориям A/B/C из `docs/architecture/improvement-recommendations.md`:**

- [ ] **Категория A (Compile-time, не модули):** `rustok-core`, `rustok-outbox`, `rustok-events`, `rustok-telemetry`, `rustok-test-utils`, `rustok-iggy`, `rustok-iggy-connector`, `rustok-mcp`, `utoipa-swagger-ui-vendored`, `tailwind-*` — НЕ имеют `impl RusToKModule`
- [x] **Категория B (Core modules):** `rustok-index`, `rustok-tenant`, `rustok-rbac` — имеют `impl RusToKModule` с `kind() -> ModuleKind::Core`
- [x] **Категория C (Optional modules):** `rustok-content`, `rustok-commerce`, `rustok-blog`, `rustok-forum`, `rustok-pages`, `alloy-scripting` — имеют `impl RusToKModule` с `kind() -> ModuleKind::Optional`
  - `rustok-content` использует default `kind()` из trait (возвращает `ModuleKind::Optional`), что корректно

### 1.4 Зависимости между crate'ами

- [x] `rustok-blog` зависит от `rustok-content` (в Cargo.toml)
- [x] `rustok-forum` зависит от `rustok-content` (в Cargo.toml)
- [ ] `rustok-index` зависит от `rustok-core` (в Cargo.toml)
- [ ] Все domain crates зависят от `rustok-core`
- [x] Нет циклических зависимостей
- [ ] `rustok-events` → `rustok-core` dependency chain корректен

---

## Фаза 2: Ядро платформы

### 2.1 rustok-core

**Путь:** `crates/rustok-core/src/`

#### Trait'ы и контракты
- [x] `RusToKModule` trait определён в `module.rs` с методами: `slug()`, `kind()`, `health()`, `dependencies()`, `migrations()`
- [x] `ModuleKind` enum имеет варианты `Core` и `Optional`
- [ ] `EventBus` trait определён
- [ ] `DomainEvent` enum содержит все нужные варианты для всех доменных модулей

#### Permissions
- [ ] `Permission` struct определён в `permissions.rs`
- [ ] `Resource` enum содержит: users, tenants, modules, settings, products, categories, orders, customers, inventory, discounts, posts, pages, nodes, media, comments, analytics, logs, webhooks, scripts
- [ ] `Action` enum содержит: create, read, update, delete, list, export, import, manage
- [ ] Роли определены: SuperAdmin, Admin, Manager, Customer
- [ ] Каждая роль имеет корректный набор permissions

#### Registry
- [x] `ModuleRegistry` в `registry.rs` разделён на `core_modules` и `optional_modules`
- [x] `register()` корректно проверяет `ModuleKind`
- [ ] `health_all()` возвращает статус всех модулей
- [ ] `toggle_module()` запрещает отключение Core-модулей

#### Security
- [ ] `SecurityContext` struct содержит `user_id`, `role`, `tenant_id`
- [ ] `PermissionScope` enum: `All`, `Own`, `None`

#### Cache
- [ ] `CacheBackend` trait определён
- [ ] `InMemoryCacheBackend` поддерживает per-entry TTL через moka
- [ ] `RedisCacheBackend` работает с CircuitBreaker
- [ ] Fallback: Redis → InMemory

#### Error handling
- [ ] `PlatformError` / типизированные ошибки определены в `error/`
- [ ] Ошибки конвертируются в HTTP-коды корректно

### 2.2 rustok-outbox

**Путь:** `crates/rustok-outbox/src/`

- [x] `TransactionalEventBus` struct определён в `transactional.rs`
- [x] Атомарная запись событий в рамках DB-транзакции
- [x] `OutboxRelay` в `relay.rs` — корректный batch processing (batch=100)
- [x] Retry policy: max retries, exponential backoff
- [ ] Transport trait: `EventTransport` в `transport.rs`
- [ ] Миграция для таблицы `sys_events` в `migration.rs`

### 2.3 rustok-events

**Путь:** `crates/rustok-events/src/`

- [ ] Re-export `DomainEvent` из `rustok-core`
- [ ] Re-export `EventEnvelope` из `rustok-core`
- [ ] Нет дублирования определений — только re-exports

### 2.4 rustok-telemetry

**Путь:** `crates/rustok-telemetry/src/`

- [ ] OpenTelemetry setup работает
- [ ] Tracing subscriber конфигурируется
- [ ] Prometheus metrics экспортируются
- [ ] Graceful shutdown для telemetry

---

## Фаза 3: Авторизация и аутентификация

### 3.1 Auth модель

**Файлы:**
- `apps/server/src/services/auth_lifecycle.rs` — единый AuthLifecycleService
- `apps/server/src/services/auth.rs` — вспомогательные auth-функции
- `apps/server/src/controllers/auth.rs` — REST auth endpoints
- `apps/server/src/graphql/auth/` — GraphQL auth resolvers
- `apps/server/src/extractors/auth.rs` — CurrentUser extractor

#### Auth lifecycle (REST ↔ GraphQL паритет)
- [ ] `register` — доступен через REST (`POST /api/auth/register`) и GraphQL (`mutation register`)
- [ ] `login/sign_in` — доступен через REST и GraphQL
- [ ] `refresh` — доступен через REST и GraphQL
- [ ] `change_password` — доступен через REST и GraphQL
- [ ] `reset_password` — доступен через REST и GraphQL (если реализован)
- [ ] Оба transport-слоя используют единый `AuthLifecycleService` (не дублируют логику)

#### JWT
- [ ] JWT generation корректен (expiry, claims)
- [ ] JWT validation работает (middleware / extractor)
- [ ] Refresh token flow реализован
- [ ] Token invalidation при смене пароля

#### Password hashing
- [ ] Используется Argon2 (crate `argon2`)
- [ ] Параметры хэширования безопасны (cost factor)
- [ ] Salt генерируется рандомно

### 3.2 CurrentUser Extractor

**Файл:** `apps/server/src/extractors/auth.rs`

- [ ] Извлекает JWT из `Authorization: Bearer <token>` header
- [ ] Декодирует claims и создаёт `CurrentUser`
- [ ] Возвращает 401 при отсутствии/невалидности токена
- [ ] `CurrentUser` содержит: id, email, role, tenant_id

---

## Фаза 4: RBAC

### 4.1 Permission Enforcement

**Файлы:**
- `apps/server/src/extractors/rbac.rs`
- `crates/rustok-core/src/permissions.rs`
- `crates/rustok-core/src/rbac.rs`

#### Extractors
- [ ] `RequireNodesCreate`, `RequireNodesRead`, `RequireNodesUpdate`, `RequireNodesDelete`, `RequireNodesList` — определены и работают
- [ ] `RequirePostsCreate`, `RequirePostsRead`, `RequirePostsUpdate`, `RequirePostsDelete`, `RequirePostsList` — определены
- [ ] `RequireProductsCreate`, `RequireProductsRead`, `RequireProductsUpdate`, `RequireProductsDelete`, `RequireProductsList` — определены
- [ ] `RequireOrdersCreate`, `RequireOrdersRead`, `RequireOrdersUpdate`, `RequireOrdersDelete`, `RequireOrdersList` — определены
- [ ] `RequireUsersCreate`, `RequireUsersRead`, `RequireUsersUpdate`, `RequireUsersDelete`, `RequireUsersList` — определены
- [ ] `RequireSettingsRead`, `RequireSettingsUpdate` — определены
- [ ] `RequireAnalyticsRead`, `RequireAnalyticsExport` — определены
- [ ] Макрос `define_permission_extractor!` работает

#### Inline checks
- [ ] `check_permission()` — проверяет одну permission
- [ ] `check_any_permission()` — проверяет OR-набор
- [ ] `check_all_permissions()` — проверяет AND-набор

#### Role-Permission матрица
- [ ] **SuperAdmin** — полный доступ ко всем ресурсам
- [ ] **Admin** — полный доступ к tenant-ресурсам, нет доступа к tenant management
- [ ] **Manager** — commerce + content CRUD, нет user management
- [ ] **Customer** — только read + own orders/comments

### 4.2 RBAC на уровне сервисов

- [ ] Все service-методы принимают `SecurityContext`
- [ ] `get_scope()` возвращает `PermissionScope::All/Own/None`
- [ ] Фильтрация по scope применяется в list-запросах (own orders для Customer)

### 4.3 RBAC на GraphQL

- [ ] GraphQL resolvers проверяют permissions перед выполнением
- [ ] Механизм проверки permissions в GraphQL context (guard / inline check)
- [ ] Ошибка 403 корректно преобразуется в GraphQL error extension

### 4.4 RBAC consistency

- [ ] Каждый REST endpoint имеет RBAC-проверку
- [ ] Каждый GraphQL mutation имеет RBAC-проверку
- [ ] Нет endpoints без auth/RBAC (кроме public: health, login, register, public storefront queries)

---

## Фаза 5: Multi-Tenancy

### 5.1 Tenant Resolution

**Файлы:**
- `apps/server/src/middleware/tenant.rs`
- `crates/rustok-tenant/src/`

- [ ] Middleware `TenantContext` извлекает tenant из: UUID header, slug header, hostname
- [ ] При отсутствии tenant → 400/404 (не 500)
- [ ] `TenantContext` доступен как Axum extractor в handlers

### 5.2 Tenant Cache

- [ ] `TenantCacheInfrastructure` хранится в `AppContext.shared_store`
- [ ] Positive cache: TTL 5 мин, capacity 1000
- [ ] Negative cache: TTL 60 сек, capacity 1000
- [ ] Versioned keys: `v1:<type>:<value>`
- [ ] Redis backend выбирается при наличии `RUSTOK_REDIS_URL`
- [ ] Fallback на InMemory при отсутствии Redis
- [ ] Stampede protection: singleflight pattern работает

### 5.3 Tenant Isolation в данных

- [ ] **Все** domain-таблицы имеют поле `tenant_id`
- [ ] **Все** SELECT-запросы в сервисах фильтруют по `tenant_id`
- [ ] **Все** INSERT-запросы проставляют `tenant_id`
- [ ] Нет cross-tenant data leaks (запрос одного tenant не видит данные другого)

### 5.4 Tenant Modules

**Таблица:** `tenant_modules`

- [ ] Таблица `tenant_modules` имеет schema: `id, tenant_id, module_slug, enabled, settings, created_at`
- [ ] UNIQUE constraint на `(tenant_id, module_slug)`
- [ ] `toggle_module()` проверяет зависимости перед отключением
- [ ] Core-модули нельзя отключить

### 5.5 Cross-instance Cache Invalidation (Redis mode)

- [ ] При обновлении tenant → PUBLISH в `tenant.cache.invalidate`
- [ ] Все инстансы подписаны и инвалидируют matching ключи
- [ ] Метрики cache hit/miss экспортируются через Redis INCR

---

## Фаза 6: Событийная система

### 6.1 Event Transport Architecture

**Файлы:**
- `apps/server/src/services/event_transport_factory.rs`
- `apps/server/src/app.rs` — `build_event_runtime()`
- `crates/rustok-outbox/src/`
- `crates/rustok-iggy/src/`

- [ ] `build_event_runtime()` вызывается в `app.rs::after_routes()`
- [ ] Transport selection: `settings.rustok.events.transport` = `memory|outbox|iggy`
- [ ] L0 (Memory): `tokio::broadcast` работает для dev
- [ ] L1 (Outbox): `TransactionalEventBus` пишет в `sys_events` таблицу
- [ ] L1 (Outbox): `OutboxRelay` читает pending events batch=100 и публикует
- [ ] L2 (Iggy): соединение с Iggy-сервером
- [ ] Default production transport = `outbox`

### 6.2 Event Flow (Write Path)

- [~] Domain service создаёт сущность + публикует DomainEvent в одной транзакции
  - [x] `rustok-content` (NodeService): корректно использует `publish_in_tx()`
  - [x] `rustok-commerce` (CatalogService, InventoryService, PricingService): корректно использует `publish_in_tx()`
  - [!] `rustok-blog` (PostService): использует `event_bus.publish()` вместо `publish_in_tx()` — нарушение атомарности
    - Файл: `crates/rustok-blog/src/services/post.rs` строки ~124, ~211, ~237, ~262, ~292
    - Риск: событие может быть потеряно если `publish()` фейлится после commit DB-транзакции в NodeService
  - [!] `rustok-forum` (TopicService, ReplyService, ModerationService): использует `event_bus.publish()` вместо `publish_in_tx()`
    - Файлы: `crates/rustok-forum/src/services/topic.rs`, `reply.rs`, `moderation.rs`
    - Те же риски потери событий
- [ ] `TransactionalEventBus::publish()` атомарно записывает в `sys_events`
- [ ] `sys_events` имеет поля: id, event_type, payload (JSON), tenant_id, status, created_at, retries
- [ ] Событие содержит `tenant_id` в payload

### 6.3 Event Flow (Read Path — Index)

- [ ] `rustok-index` подписывается на DomainEvents
- [ ] При `NodeCreated/NodeUpdated` — обновляет `index_content`
- [ ] При `ProductCreated/ProductUpdated` — обновляет `index_products`
- [ ] Indexer корректно обрабатывает ошибки (не теряет события)

### 6.4 DomainEvent Coverage

**Проверяем, что каждый модуль публикует нужные события:**

#### Content Events
- [ ] `NodeCreated` — при создании node
- [ ] `NodeUpdated` — при обновлении node
- [ ] `NodeDeleted` — при удалении node
- [ ] `NodePublished` / `NodeUnpublished` — при смене статуса

#### Commerce Events
- [ ] `ProductCreated`, `ProductUpdated`, `ProductDeleted`
- [ ] `VariantCreated`, `VariantUpdated`
- [ ] `PriceUpdated`
- [ ] `InventoryUpdated`
- [ ] `OrderPlaced`, `OrderStatusChanged`

#### Blog Events
- [ ] `PostCreated`, `PostUpdated`, `PostDeleted`
- [ ] `PostPublished`, `PostUnpublished`

#### Forum Events
- [ ] `TopicCreated`, `TopicUpdated`
- [ ] `ReplyCreated`

#### Pages Events
- [ ] `PageCreated`, `PageUpdated`, `PageDeleted`
- [ ] `PagePublished`

### 6.5 Outbox Relay

- [x] Relay обрабатывает pending events корректно
- [x] Retry с exponential backoff (1s → 60s)
- [x] Max retries = 5 (или конфигурируемо)
- [x] DLQ: после max retries → status = `failed`
- [x] Метрики: `outbox_backlog_size`, `outbox_retries_total`, `outbox_dlq_total`

### 6.6 Event Versioning

- [ ] Каждый DomainEvent имеет версию (или version field в envelope)
- [ ] Обратная совместимость при добавлении новых полей

---

## Фаза 7: Доменные модули

### 7.1 rustok-content

**Путь:** `crates/rustok-content/`

#### Entities
- [x] `nodes` entity: id, tenant_id, slug, node_type, status, author_id, created_at, updated_at
- [x] `node_translations` entity: id, node_id, locale, title, body
- [ ] `bodies` entity (если отдельная)
- [x] Все entities имеют tenant_id

#### Services
- [x] `NodeService` — CRUD для nodes
- [x] `create_node()` — валидация, сохранение, публикация `NodeCreated`
- [x] `update_node()` — валидация, сохранение, публикация `NodeUpdated`
- [ ] `delete_node()` — soft/hard delete, публикация `NodeDeleted`
- [x] `list_nodes()` — пагинация, фильтрация, tenant_id scope
- [x] `publish_node()` / `unpublish_node()` — state machine transition

#### DTOs
- [x] `CreateNodeInput` — валидация полей
- [x] `UpdateNodeInput` — partial update
- [x] `NodeResponse` — response DTO
- [x] `NodeListItem` — list response DTO

#### State Machine
- [x] Node status transitions: Draft → Published → Archived
- [x] Невалидные transitions возвращают ошибку
- [x] Property tests для state machine (`state_machine_proptest.rs`)

#### Migrations
- [ ] Миграция создаёт таблицу `nodes`
- [ ] Миграция создаёт таблицу `node_translations`
- [~] Миграции доступны через `RusToKModule::migrations()`
  - Примечание: `ContentModule::migrations()` возвращает `Vec::new()` — миграции обрабатываются главным приложением

### 7.2 rustok-commerce

**Путь:** `crates/rustok-commerce/`

#### Entities
- [ ] `products` entity: id, tenant_id, slug, status, created_at, updated_at
- [ ] `product_translations` entity
- [ ] `product_variants` entity: id, product_id, sku, price, stock
- [ ] `variant_translations` entity
- [ ] `prices` entity: id, variant_id, currency, amount
- [ ] `product_images` entity
- [ ] `product_options` entity
- [ ] Все entities имеют tenant_id (или наследуют через product)

#### Services
- [x] `CatalogService` — CRUD для products
- [x] `InventoryService` — управление стоками
- [x] `PricingService` — управление ценами
- [x] Все сервисы используют `TransactionalEventBus` (корректно через `publish_in_tx()`)
- [x] Все сервисы принимают `SecurityContext`

#### DTOs
- [ ] `CreateProductInput` / `UpdateProductInput`
- [ ] `ProductResponse` / `ProductListItem`
- [ ] `CreateVariantInput` / `VariantResponse`

#### State Machine
- [ ] Product status: Draft → Active → Archived
- [ ] Property tests для state machine

#### Migrations
- [ ] Все commerce-таблицы имеют миграции
- [ ] Миграции доступны через `RusToKModule::migrations()`

### 7.3 rustok-blog

**Путь:** `crates/rustok-blog/`

- [x] Зависит от `rustok-content` (uses nodes internally)
- [x] `BlogModule::dependencies()` возвращает `&["content"]`
- [x] `PostService` — CRUD для постов (обёртка над NodeService)
- [x] State machine: Draft → Published → Archived
- [!] Events: `PostCreated`, `PostPublished`, etc. — **публикуются через `event_bus.publish()` без транзакции**
  - `PostService` передаёт `event_bus` в `NodeService` (который использует `publish_in_tx()`),
    но сам дополнительно вызывает `self.event_bus.publish()` для Blog-специфичных событий вне транзакции
  - Затронутые вызовы: post.rs строки ~124, ~211, ~237, ~262, ~292
- [x] DTOs: `CreatePostInput`, `PostResponse`, `PostListItem`
- [x] Поддержка i18n (locale.rs)
- [ ] Миграции

### 7.4 rustok-forum

**Путь:** `crates/rustok-forum/`

- [x] Зависит от `rustok-content`
- [x] `ForumModule::dependencies()` возвращает `&["content"]`
- [x] `TopicService` — CRUD для тем
- [x] `ReplyService` — CRUD для ответов
- [x] `CategoryService` — категории форума
- [!] Events: `TopicCreated`, `ReplyCreated`, etc. — **публикуются через `event_bus.publish()` без транзакции**
  - Затронутые файлы: `topic.rs`, `reply.rs`, `moderation.rs` (строки ~97, ~187, ~228, ~296)
- [x] DTOs: `CreateTopicInput`, `TopicResponse`, etc.
- [x] Поддержка i18n (locale.rs)
- [ ] Миграции
- [x] Constants (`constants.rs`)

### 7.5 rustok-pages

**Путь:** `crates/rustok-pages/`

- [ ] `PageService` — CRUD для страниц
- [ ] State machine: Draft → Published → Archived
- [ ] Events: `PageCreated`, `PagePublished`, etc.
- [ ] DTOs: `CreatePageInput`, `PageResponse`
- [ ] Миграции

### 7.6 alloy-scripting

**Путь:** `crates/alloy-scripting/`

- [ ] `AlloyModule` зарегистрирован как `ModuleKind::Optional` (в `apps/server/src/modules/alloy.rs`)
- [ ] Rhai scripting engine инициализируется
- [ ] `scripts` таблица — CRUD для скриптов
- [ ] RBAC permissions: `Scripts` resource (create/read/update/delete/list/manage)
- [ ] Безопасность: скрипты не могут выполнять произвольный I/O
- [ ] Миграции

### 7.7 rustok-index (CQRS Read Models)

**Путь:** `crates/rustok-index/`

- [ ] `IndexModule` зарегистрирован как `ModuleKind::Core`
- [ ] Content indexer: слушает content events → пишет в `index_content`
- [ ] Product indexer: слушает commerce events → пишет в `index_products`
- [ ] Denormalized models для fast reads
- [ ] Поисковые сервисы (search)
- [ ] Engine trait (`engine.rs`, `pg_engine.rs`)
- [ ] Listener pattern (`listener.rs`)

### 7.8 rustok-rbac

**Путь:** `crates/rustok-rbac/`

- [ ] `RbacModule` зарегистрирован как `ModuleKind::Core`
- [ ] Entities, DTOs, Services
- [ ] Health check работает
- [ ] Миграции

### 7.9 rustok-tenant

**Путь:** `crates/rustok-tenant/`

- [ ] `TenantModule` зарегистрирован как `ModuleKind::Core`
- [ ] Entities: `tenants`, `tenant_modules`
- [ ] Services: CRUD для tenants, module toggle
- [ ] Health check работает
- [ ] Миграции

---

## Фаза 8: API — GraphQL

### 8.1 Schema Assembly

**Файлы:**
- `apps/server/src/graphql/schema.rs`
- `apps/server/src/graphql/queries.rs`
- `apps/server/src/graphql/mutations.rs`

- [ ] Schema собирается через `MergedObject`
- [ ] `RootQuery` содержит: `ContentQuery`, `CommerceQuery`, `BlogQuery`, `ForumQuery`, `AlloyQuery`, `PagesQuery` (если есть)
- [ ] `RootMutation` содержит: `ContentMutation`, `CommerceMutation`, `BlogMutation`, `ForumMutation`, `AlloyMutation`, `PagesMutation` (если есть)
- [ ] Schema endpoint: `POST /api/graphql`
- [ ] GraphQL Playground / IDE доступен (если включён)

### 8.2 Content GraphQL

**Файлы:** `apps/server/src/graphql/content/`

- [ ] Query: `node(id)`, `nodes(filter, pagination)`
- [ ] Mutation: `createNode`, `updateNode`, `deleteNode`, `publishNode`
- [ ] Auth/RBAC проверяются
- [ ] Tenant isolation соблюдается

### 8.3 Commerce GraphQL

**Файлы:** `apps/server/src/graphql/commerce/`

- [ ] Query: `product(id)`, `products(filter)`, `order(id)`, `orders(filter)`
- [ ] Mutation: `createProduct`, `updateProduct`, `deleteProduct`, `createOrder`
- [ ] Variants: `addVariant`, `updateVariant`
- [ ] Inventory: `updateStock`
- [ ] Prices: `updatePrice`

### 8.4 Blog GraphQL

**Файлы:** `apps/server/src/graphql/blog/`

- [ ] Query: `post(id)`, `posts(filter)`
- [ ] Mutation: `createPost`, `updatePost`, `deletePost`, `publishPost`

### 8.5 Forum GraphQL

**Файлы:** `apps/server/src/graphql/forum/`

- [ ] Query: `topic(id)`, `topics(filter)`, `replies(topicId)`
- [ ] Mutation: `createTopic`, `createReply`, `updateTopic`
- [ ] Categories: query/mutation

### 8.6 Alloy GraphQL

**Файлы:** `apps/server/src/graphql/alloy/`

- [ ] Query: `script(id)`, `scripts(filter)`
- [ ] Mutation: `createScript`, `updateScript`, `executeScript`, `deleteScript`

### 8.7 Pages GraphQL

**Файлы:** `apps/server/src/graphql/pages/`

- [ ] Query: `page(id/slug)`, `pages(filter)`
- [ ] Mutation: `createPage`, `updatePage`, `deletePage`, `publishPage`

### 8.8 DataLoader

**Файл:** `apps/server/src/graphql/loaders.rs`

- [ ] DataLoaders определены для N+1 prevention
- [ ] Используются в resolvers для связанных данных

### 8.9 Auth GraphQL

**Файлы:** `apps/server/src/graphql/auth/`

- [ ] `register` mutation
- [ ] `login` / `signIn` mutation
- [ ] `refreshToken` mutation
- [ ] `changePassword` mutation
- [ ] `me` query (current user info)

### 8.10 Observability GraphQL

**Файл:** `apps/server/src/graphql/observability.rs`

- [ ] Query для module health
- [ ] Query для system status

---

## Фаза 9: API — REST

### 9.1 Auth REST

**Файл:** `apps/server/src/controllers/auth.rs`

- [ ] `POST /api/auth/register` — регистрация
- [ ] `POST /api/auth/login` — вход
- [ ] `POST /api/auth/refresh` — обновление токена
- [ ] `POST /api/auth/change-password` — смена пароля
- [ ] `POST /api/auth/reset-password` — сброс пароля (если есть)

### 9.2 Health REST

**Файл:** `apps/server/src/controllers/health.rs`

- [ ] `GET /api/health` — общий health check
- [ ] Включает статус модулей из `ModuleRegistry::health_all()`
- [ ] Включает статус DB-соединения
- [ ] Возвращает 200 OK / 503 Service Unavailable

### 9.3 Commerce REST

**Файлы:** `apps/server/src/controllers/commerce/`

- [ ] `products.rs` — CRUD для products (REST)
- [ ] `variants.rs` — управление вариантами
- [ ] `inventory.rs` — управление стоками
- [ ] Все endpoints имеют RBAC
- [ ] Все endpoints имеют tenant isolation
- [ ] OpenAPI annotations (`#[utoipa::path(...)]`)

### 9.4 Content REST

**Файлы:** `apps/server/src/controllers/content/`

- [ ] `nodes.rs` — CRUD для nodes
- [ ] RBAC + tenant isolation

### 9.5 Blog REST

**Файлы:** `apps/server/src/controllers/blog/`

- [ ] `posts.rs` — CRUD для posts
- [ ] RBAC + tenant isolation

### 9.6 Forum REST

**Файлы:** `apps/server/src/controllers/forum/`

- [ ] `topics.rs` — CRUD для topics
- [ ] `replies.rs` — CRUD для replies
- [ ] `categories.rs` — CRUD для categories
- [ ] RBAC + tenant isolation

### 9.7 Pages REST

**Файл:** `apps/server/src/controllers/pages.rs`

- [ ] CRUD для pages
- [ ] RBAC + tenant isolation

### 9.8 Admin Events REST

**Файл:** `apps/server/src/controllers/admin_events.rs`

- [ ] `GET /api/admin/events/dlq` — просмотр DLQ
- [ ] `POST /api/admin/events/dlq/{id}/replay` — replay
- [ ] Только для SuperAdmin/Admin

### 9.9 Metrics & Swagger

- [ ] `GET /metrics` — Prometheus metrics (`controllers/metrics.rs`)
- [ ] `GET /swagger` — Swagger UI (`controllers/swagger.rs`)
- [ ] OpenAPI spec генерируется корректно через `utoipa`

### 9.10 Rate Limiting

**Файл:** `apps/server/src/middleware/rate_limit.rs`

- [ ] Rate limiting middleware подключён
- [ ] Корректные лимиты для auth endpoints (login/register)
- [ ] Корректные лимиты для API endpoints

---

## Фаза 10: Фронтенды — Leptos

### 10.1 apps/admin (Leptos CSR)

**Путь:** `apps/admin/`

- [ ] Cargo.toml: зависимости корректны (leptos, leptos-auth, leptos-graphql, iu-leptos, etc.)
- [ ] Собирается: `cargo build -p rustok-admin`
- [ ] Entry point: `main.rs` / `lib.rs`
- [ ] Routing: все admin-страницы доступны
- [ ] Auth: login page → JWT хранение → authenticated requests
- [ ] GraphQL client: подключение к `/api/graphql`
- [ ] Используется `leptos-auth` для auth state
- [ ] Используется `leptos-zustand` для state management
- [ ] Используется `leptos-graphql` для GraphQL queries/mutations
- [ ] Используется `iu-leptos` (IU компоненты) для UI

#### Страницы admin panel
- [ ] Dashboard (главная)
- [ ] Products list / create / edit
- [ ] Orders list / view
- [ ] Content / Nodes list / create / edit
- [ ] Blog posts list / create / edit
- [ ] Pages list / create / edit
- [ ] Users management
- [ ] Settings
- [ ] Module management (toggle per-tenant)

### 10.2 apps/storefront (Leptos SSR)

**Путь:** `apps/storefront/`

- [ ] Собирается
- [ ] SSR работает (server-side rendering)
- [ ] SEO: meta tags, structured data
- [ ] Product catalog page
- [ ] Product detail page
- [ ] Blog posts page
- [ ] Static pages
- [ ] Cart / Checkout flow (если реализован)

---

## Фаза 11: Фронтенды — Next.js

### 11.1 apps/next-admin

**Путь:** `apps/next-admin/`

- [ ] `package.json`: зависимости корректны
- [ ] `npm install` проходит
- [ ] `npm run build` проходит
- [ ] `npm run lint` проходит
- [ ] Clerk auth setup (`docs/clerk_setup.md`)
- [ ] RBAC навигация (`docs/nav-rbac.md`)
- [ ] Темизация (`docs/themes.md`)
- [ ] GraphQL клиент подключён и работает
- [ ] Используются packages из `packages/` (leptos-auth, leptos-graphql, etc.)
- [ ] Routing: все admin-страницы доступны
- [ ] TypeScript компилируется без ошибок

### 11.2 apps/next-frontend

**Путь:** `apps/next-frontend/`

- [ ] `package.json`: зависимости корректны
- [ ] `npm install && npm run build` проходит
- [ ] SSR / SSG для SEO
- [ ] Product catalog
- [ ] Blog
- [ ] Static pages
- [ ] TypeScript компилируется без ошибок

---

## Фаза 12: Фронтенд-библиотеки

### 12.1 Leptos-библиотеки (crates/)

Для каждой проверяем: компилируется, exports корректны, используется в apps/admin или apps/storefront.

- [ ] `leptos-auth` — auth state management, JWT storage
- [ ] `leptos-graphql` — GraphQL client wrapper
- [ ] `leptos-hook-form` — form handling
- [ ] `leptos-forms` — form components
- [ ] `leptos-zod` — validation schemas
- [ ] `leptos-zustand` — state management (Zustand-like)
- [ ] `leptos-ui` — UI wrappers
- [ ] `leptos-table` — table component
- [ ] `leptos-shadcn-pagination` — pagination component

### 12.2 IU (Design System)

- [ ] `UI/leptos` — Leptos компоненты (iu-leptos crate)
- [ ] `UI/next/components` — React/Next.js компоненты
- [ ] Компоненты между Leptos и Next.js визуально согласованы (если applicable)
- [ ] `UI/docs/api-contracts.md` актуален

### 12.3 TypeScript packages (packages/)

Для каждого: npm install, npm build, npm test (если есть), используется в apps/next-*.

- [ ] `packages/leptos-auth` — TypeScript auth helpers
- [ ] `packages/leptos-graphql` — TypeScript GraphQL helpers
- [ ] `packages/leptos-hook-form` — TypeScript form helpers
- [ ] `packages/leptos-zod` — TypeScript validation
- [ ] `packages/leptos-zustand` — TypeScript state management

---

## Фаза 13: Интеграционные связи

### 13.1 Write → Event → Index (E2E data flow)

- [ ] Admin создаёт Product → `ProductCreated` event → Index обновляет `index_products` → Storefront видит product
- [ ] Admin создаёт Node → `NodeCreated` event → Index обновляет `index_content` → Storefront видит content
- [ ] Admin создаёт Post → `PostCreated` event → Index обновляет → Storefront видит post

### 13.2 Module Dependencies

- [ ] При отключении `content` модуля → `blog` и `forum` автоматически отключаются (или ошибка)
- [ ] При включении `blog` → `content` должен быть включён (или ошибка)
- [ ] Core modules не могут быть отключены ни при каких условиях

### 13.3 Frontend ↔ Backend API Contracts

- [ ] GraphQL schema, используемая фронтендами, совпадает с реальной серверной schema
- [ ] REST endpoints, вызываемые фронтендами, существуют на сервере
- [ ] Типы данных (DTOs) между фронтом и бэком согласованы
- [ ] Auth token format одинаков для всех фронтендов

### 13.4 Leptos Admin ↔ Server

- [ ] Admin использует `/api/graphql` endpoint
- [ ] Auth flow: login → получить JWT → хранить в localStorage/cookie → отправлять в headers
- [ ] RBAC: admin видит только разрешённые разделы

### 13.5 Next.js Admin ↔ Server

- [ ] Next Admin использует тот же `/api/graphql` endpoint
- [ ] Clerk auth ↔ Server JWT: механизм интеграции работает
- [ ] RBAC навигация на фронте совпадает с серверным RBAC

### 13.6 Storefront ↔ Server

- [ ] Storefront (Leptos SSR) получает данные из index tables через GraphQL
- [ ] Next.js storefront получает те же данные
- [ ] Public queries не требуют auth
- [ ] Корзина/заказы требуют auth

---

## Фаза 14: Тестовое покрытие

### 14.1 Unit-тесты (cargo test --lib)

Для каждого crate проверяем наличие и прохождение unit-тестов:

- [ ] `rustok-core` — `cargo test -p rustok-core --lib`
- [ ] `rustok-content` — `cargo test -p rustok-content --lib`
- [ ] `rustok-commerce` — `cargo test -p rustok-commerce --lib`
- [ ] `rustok-blog` — `cargo test -p rustok-blog --lib`
- [ ] `rustok-forum` — `cargo test -p rustok-forum --lib`
- [ ] `rustok-pages` — `cargo test -p rustok-pages --lib`
- [ ] `rustok-index` — `cargo test -p rustok-index --lib`
- [ ] `rustok-tenant` — `cargo test -p rustok-tenant --lib`
- [ ] `rustok-rbac` — `cargo test -p rustok-rbac --lib`
- [ ] `rustok-outbox` — `cargo test -p rustok-outbox --lib`
- [ ] `rustok-telemetry` — `cargo test -p rustok-telemetry --lib`
- [ ] `alloy-scripting` — `cargo test -p alloy-scripting --lib`

### 14.2 Integration тесты

**Файлы в `tests/` директориях:**

#### Server integration tests (`apps/server/tests/`)
- [ ] `integration/content_flow_test.rs` — проходит
- [ ] `integration/event_flow_test.rs` — проходит
- [ ] `integration/order_flow_test.rs` — проходит
- [ ] `multi_tenant_isolation_test.rs` — проходит
- [ ] `tenant_cache_stampede_test.rs` — проходит

#### Crate integration tests
- [ ] `rustok-commerce/tests/catalog_service_test.rs`
- [ ] `rustok-commerce/tests/inventory_service_test.rs`
- [ ] `rustok-commerce/tests/pricing_service_test.rs`
- [ ] `rustok-commerce/tests/product_event_index_integration_test.rs`
- [ ] `rustok-content/tests/node_service_test.rs`
- [ ] `rustok-content/tests/node_event_index_integration_test.rs`
- [ ] `rustok-core/tests/event_versioning_test.rs`
- [ ] `rustok-core/tests/security_audit_test.rs`
- [ ] `rustok-core/tests/transactional_events_integration_test.rs`
- [ ] `rustok-core/tests/transactional_events_test.rs`
- [ ] `rustok-outbox/tests/sqlite_transport_regression_test.rs`
- [ ] `rustok-telemetry/tests/metrics_test.rs`
- [ ] `rustok-telemetry/tests/otel_test.rs`
- [ ] `rustok-blog/tests/` — все тесты
- [ ] `rustok-forum/tests/` — все тесты
- [ ] `rustok-pages/tests/` — все тесты
- [ ] `rustok-index/tests/` — все тесты
- [ ] `rustok-rbac/tests/` — все тесты
- [ ] `rustok-tenant/tests/` — все тесты
- [ ] `rustok-iggy/tests/` — все тесты
- [ ] `rustok-mcp/tests/` — все тесты

### 14.3 Property-based тесты (proptest)

- [ ] `rustok-content/src/state_machine_proptest.rs` — проходит
- [ ] `rustok-commerce/src/state_machine_proptest.rs` — проходит
- [ ] `rustok-blog/src/state_machine_proptest.rs` — проходит
- [ ] `rustok-core/src/validation_proptest.rs` — проходит

### 14.4 Security audit тесты

- [ ] `rustok-core/tests/security_audit_test.rs` — проходит
- [ ] Проверка на SQL injection
- [ ] Проверка на XSS в input validation
- [ ] Проверка auth bypass scenarios

### 14.5 Benchmarks

**Путь:** `benches/`

- [ ] `cargo bench` компилируется
- [ ] Benchmark results адекватны
- [ ] Нет performance regressions

### 14.6 Полный тестовый прогон

- [ ] `cargo test --workspace` — ВСЕ тесты проходят (или документированы known failures)
- [ ] Нет flaky тестов
- [ ] Тесты не зависят от порядка выполнения

---

## Фаза 15: Observability и операционная готовность

### 15.1 Prometheus Metrics

- [ ] `/metrics` endpoint доступен
- [ ] HTTP request metrics (count, latency, status)
- [ ] Database metrics (pool size, query time)
- [ ] Cache metrics (hit/miss)
- [ ] Outbox metrics (backlog_size, retries, dlq)
- [ ] Module health metrics
- [ ] Custom business metrics (если есть)

### 15.2 Tracing / OpenTelemetry

- [ ] `rustok-telemetry` настраивает tracing subscriber
- [ ] Span'ы создаются для HTTP requests
- [ ] Span'ы создаются для DB queries
- [ ] Span'ы создаются для event processing
- [ ] OTLP exporter конфигурируется (если включён)

### 15.3 Health Checks

- [ ] `GET /api/health` возвращает JSON с module statuses
- [ ] DB health check
- [ ] Redis health check (если Redis включён)
- [ ] Module health checks от каждого зарегистрированного модуля

### 15.4 Grafana Dashboards

**Путь:** `grafana/`

- [ ] Dashboard JSON файлы валидны
- [ ] Dashboards покрывают: HTTP, DB, Cache, Events
- [ ] Grafana datasource конфигурация корректна

### 15.5 Docker Compose

- [ ] `docker-compose.yml` — минимальный dev setup (PostgreSQL, Redis)
- [ ] `docker-compose.full-dev.yml` — полный dev setup
- [ ] `docker-compose.observability.yml` — Prometheus + Grafana
- [ ] Все services стартуют без ошибок
- [ ] Порты не конфликтуют

---

## Фаза 16: Синхронизация документации с кодом

### 16.1 docs/architecture/

- [ ] `overview.md` — соответствует текущей архитектуре (модули, data flow, CQRS)
- [ ] `api.md` — GraphQL/REST endpoints актуальны
- [ ] `rbac.md` — permissions, roles, extractors совпадают с кодом
- [ ] `tenancy.md` — cache layers, stampede protection, invalidation описаны корректно
- [ ] `events.md` — транспортные уровни (L0/L1/L2) описаны корректно
- [ ] `events-transactional.md` — TransactionalEventBus описан
- [ ] `event-flow-contract.md` — контракт событий актуален
- [ ] `database.md` — schema совпадает с миграциями
- [ ] `modules.md` — перечень модулей актуален
- [ ] `routing.md` — policy GraphQL vs REST актуальна
- [ ] `dataloader.md` — описание DataLoader актуально
- [ ] `improvement-recommendations.md` — статусы рекомендаций актуальны

### 16.2 docs/guides/

- [ ] `quickstart.md` — инструкции по запуску работают
- [ ] `testing.md` — гайд по тестированию актуален
- [ ] `testing-integration.md` — примеры integration тестов работают
- [ ] `testing-property.md` — примеры property тестов работают
- [ ] `error-handling.md` — паттерн ошибок совпадает с кодом
- [ ] `input-validation.md` — валидация описана корректно
- [ ] `rate-limiting.md` — конфигурация rate limiting актуальна
- [ ] `security-audit.md` — описание совпадает с тестами
- [ ] `observability-quickstart.md` — настройка метрик/трейсинга работает

### 16.3 Module docs

Для каждого модуля проверяем: README и docs/README совпадают с реальным состоянием кода.

- [ ] `rustok-core` — README/docs соответствуют API
- [ ] `rustok-content` — README/docs/implementation-plan актуальны
- [ ] `rustok-commerce` — README/docs/implementation-plan актуальны
- [ ] `rustok-blog` — README/docs/implementation-plan актуальны
- [ ] `rustok-forum` — README/docs/implementation-plan актуальны
- [ ] `rustok-pages` — README/docs/implementation-plan актуальны
- [ ] `rustok-index` — README/docs/implementation-plan актуальны
- [ ] `rustok-tenant` — README/docs/implementation-plan актуальны
- [ ] `rustok-rbac` — README/docs/implementation-plan актуальны
- [ ] `rustok-outbox` — README/docs/implementation-plan актуальны
- [ ] `rustok-telemetry` — README/docs/implementation-plan актуальны
- [ ] `rustok-iggy` — README/docs/implementation-plan актуальны
- [ ] `rustok-iggy-connector` — README/docs/implementation-plan актуальны
- [ ] `alloy-scripting` — README/docs актуальны
- [ ] `rustok-events` — README актуален
- [ ] `rustok-mcp` — README/docs актуальны

### 16.4 Frontend library docs

- [ ] `leptos-auth` — README/docs совпадают с API
- [ ] `leptos-graphql` — README/docs совпадают
- [ ] `leptos-hook-form` — README/docs совпадают
- [ ] `leptos-zod` — README/docs совпадают
- [ ] `leptos-zustand` — README/docs совпадают
- [ ] `leptos-table` — README/docs совпадают
- [ ] `leptos-shadcn-pagination` — README/docs совпадают

### 16.5 Application docs

- [ ] `apps/server/docs/README.md` — актуален
- [ ] `apps/admin/docs/README.md` — актуален
- [ ] `apps/storefront/README.md` — актуален
- [ ] `apps/next-admin/README.md` — актуален
- [ ] `apps/next-frontend/docs/README.md` — актуален

### 16.6 Root-level docs

- [ ] `README.md` — описание проекта актуально
- [ ] `PLATFORM_INFO_RU.md` — информация актуальна
- [ ] `RUSTOK_MANIFEST.md` — манифест актуален
- [ ] `AGENTS.md` — правила для AI-агентов актуальны
- [ ] `CONTRIBUTING.md` — инструкции для контрибьюторов актуальны
- [ ] `CHANGELOG.md` — последние изменения задокументированы
- [ ] `docs/index.md` — карта документации полная и актуальная

### 16.7 ADR (Architectural Decision Records)

**Путь:** `DECISIONS/`

- [ ] Все ADR в `DECISIONS/` имеют корректный формат
- [ ] Статусы ADR актуальны (Accepted / Superseded / etc.)
- [ ] ADR для event-contract migration существует
- [ ] ADR для core-server module-bundles routing существует
- [ ] ADR для auth lifecycle unification существует

---

## Фаза 17: CI/CD и DevOps

### 17.1 GitHub Actions

**Файлы:** `.github/workflows/`

- [ ] `ci.yml` — основной CI pipeline
  - [ ] Запускает `cargo check`
  - [ ] Запускает `cargo test`
  - [ ] Запускает `cargo clippy`
  - [ ] Запускает `cargo fmt --check`
  - [ ] Запускает frontend builds (если applicable)
- [ ] `dependencies.yml` — проверка зависимостей (Dependabot / cargo-deny)

### 17.2 cargo-deny

**Файл:** `deny.toml`

- [ ] `cargo deny check` проходит
- [ ] Лицензии разрешены
- [ ] Нет banned crates
- [ ] Нет known advisories

### 17.3 Typos

**Файл:** `typos.toml`

- [ ] `typos` check проходит (нет опечаток в коде)

### 17.4 Scripts

**Путь:** `scripts/`

- [ ] Скрипты актуальны и работают
- [ ] Описаны в README или Makefile

---

## Фаза 18: Безопасность

### 18.1 Authentication Security

- [ ] Пароли хэшируются Argon2 (не plain text, не MD5/SHA)
- [ ] JWT secret хранится в конфигурации (не в коде)
- [ ] Token expiry разумный (access: 15-60 мин, refresh: дни)
- [ ] Refresh token rotation (старый инвалидируется при использовании)

### 18.2 Authorization Security

- [ ] Нет endpoint'ов без auth (кроме public)
- [ ] RBAC extractors используются последовательно (не пропущены)
- [ ] SuperAdmin endpoints недоступны обычным пользователям

### 18.3 Tenant Isolation Security

- [ ] Нет SQL-запросов без `WHERE tenant_id = ?`
- [ ] Нет GraphQL resolvers, возвращающих данные без tenant filter
- [ ] Тест на cross-tenant access существует и проходит

### 18.4 Input Validation

- [ ] Все CreateInput/UpdateInput DTOs используют `validator`
- [ ] SQL injection невозможен (параметризованные запросы через SeaORM)
- [ ] XSS: пользовательский ввод экранируется
- [ ] CORS настроен корректно (`tower-http::cors`)

### 18.5 Secrets Management

- [ ] `.env.dev.example` не содержит реальных secrets
- [ ] `.gitignore` исключает `.env`, credentials, keys
- [ ] JWT secret, DB password, Redis password — через env vars

### 18.6 Dependency Security

- [ ] `cargo audit` не показывает critical vulnerabilities
- [ ] `cargo deny check advisories` проходит

---

## Фаза 19: Антипаттерны и качество кода

**Справочные документы:**
- [Паттерны vs Антипаттерны](./standards/patterns-vs-antipatterns.md)
- [Запрещённые действия](./standards/forbidden-actions.md)
- [Known Pitfalls](./ai/KNOWN_PITFALLS.md)

### 19.1 Критические антипаттерны (MUST FIX)

Поиск запрещённых паттернов в production коде. Каждый найденный экземпляр — обязательное исправление.

#### Tenant Isolation violations
- [ ] Поиск `find().all(&db)` без `.filter(...tenant_id...)` в domain crates
  - `grep -rn "\.all(&" crates/rustok-content/src/ crates/rustok-commerce/src/ crates/rustok-blog/src/ crates/rustok-forum/src/ crates/rustok-pages/src/`
- [ ] Поиск `find_by_id` без tenant_id проверки
- [ ] Поиск DELETE без tenant_id filter
- [ ] Проверка: каждая domain-таблица имеет `tenant_id` column в миграции
- [ ] Проверка: каждый SeaORM entity имеет `tenant_id` поле

#### Unsafe event publishing
- [!] Поиск `publish(` без `_in_tx` в domain services
  - `grep -rn "event_bus\.publish(" crates/rustok-*/src/services/` — найдены нарушения:
    - `crates/rustok-blog/src/services/post.rs` — 5 вызовов `event_bus.publish()` вместо `publish_in_tx()`
    - `crates/rustok-forum/src/services/moderation.rs` — 3 вызова
    - `crates/rustok-forum/src/services/reply.rs` — 1 вызов
    - `crates/rustok-forum/src/services/topic.rs` — 1 вызов (по паттерну)
  - Требует исправления: заменить на `publish_in_tx()` с передачей транзакции
- [ ] Проверка: каждый DomainEvent в crates содержит `tenant_id` field

#### Hardcoded secrets
- [ ] Поиск hardcoded passwords/secrets/keys в Rust коде
  - `grep -rn "password\|secret\|api_key" --include="*.rs" | grep -v test | grep -v "// " | grep "="`
- [ ] Поиск в .ts/.tsx файлах
- [ ] Проверка: `.env` файлы отсутствуют в git (только `.env.dev.example`)

#### Panics в production
- [ ] Поиск `unwrap()` в production коде (исключая tests)
  - `grep -rn "\.unwrap()" crates/rustok-*/src/ apps/server/src/ --include="*.rs" | grep -v "#\[cfg(test)\]" | grep -v "mod tests"`
- [ ] Поиск `expect()` в production коде (проверить каждый: оправдан или нет)
  - `grep -rn "\.expect(" crates/rustok-*/src/ apps/server/src/ --include="*.rs" | grep -v test`
- [ ] Поиск `panic!` в production коде

### 19.2 RBAC coverage audit

- [ ] Список ВСЕХ handlers в `apps/server/src/controllers/` — каждый имеет RBAC extractor или явно помечен как public
- [ ] Список ВСЕХ GraphQL mutations — каждый имеет permission check
- [ ] Список ВСЕХ GraphQL queries (non-public) — каждый имеет permission check
- [ ] Проверка: нет `CurrentUser` без RBAC check (кроме public endpoints)

### 19.3 Async safety

- [x] Поиск `std::thread::sleep` в async коде — не найдено в production коде (только в тестах)
- [~] Поиск `std::fs::` в async коде
  - `apps/server/src/tasks/cleanup.rs` — `std::fs::` используется в task коде (приемлемо, не в HTTP handlers)
- [ ] Поиск неограниченных `tokio::spawn` в циклах без Semaphore
- [x] Проверка: нет `block_on()` внутри async context

### 19.4 Error handling quality

- [x] Все domain crates используют `thiserror` (не `anyhow` в lib)
- [x] Нет `String` errors — типизированные ошибки
- [ ] Controllers используют `loco_rs::Result` (не custom error types)
- [ ] GraphQL resolvers корректно конвертируют errors в extensions

### 19.5 Code metrics

- [ ] Нет функций > 40 строк (проверить top-10 самых длинных функций)
- [ ] Нет модулей > 1000 строк
- [ ] Нет функций с > 6 аргументами
- [ ] `DomainEvent` enum не превышает разумного размера (проверить `rustok-core/src/events/types.rs`)

### 19.6 State machine correctness

- [ ] Каждый state machine модуль имеет `*_proptest.rs`
  - `rustok-content/src/state_machine_proptest.rs`
  - `rustok-commerce/src/state_machine_proptest.rs`
  - `rustok-blog/src/state_machine_proptest.rs`
- [ ] Невалидные переходы возвращают ошибку (не panic)
- [ ] Нет string-based status checks (`if status == "published"`)

### 19.7 DTO consistency

- [ ] Каждый domain module имеет отдельные `CreateInput`/`UpdateInput`/`Response` DTOs
- [ ] Entities из БД НЕ возвращаются напрямую в API
- [ ] DTOs имеют `#[derive(Validate)]` для input validation
- [ ] Response DTOs не содержат internal fields (например, `password_hash`)

### 19.8 Event handling quality

- [ ] Каждый event handler idempotent (retry-safe)
- [ ] Каждый новый DomainEvent имеет integration test
- [ ] Каждый event handler имеет idempotency test
- [ ] Event payloads содержат все необходимые поля для обработки (не требуют дополнительных DB queries)

### 19.9 Frontend antipatterns

#### Leptos
- [ ] Нет прямых `fetch()` — используется `leptos-graphql`
- [ ] Нет ручного JWT management — используется `leptos-auth`
- [ ] Нет prop drilling > 3 уровней — используется `leptos-zustand`
- [ ] Admin — CSR (WASM), Storefront — SSR

#### Next.js
- [ ] Нет `any` типов в TypeScript коде
  - `grep -rn ": any" apps/next-admin/src/ apps/next-frontend/src/ --include="*.ts" --include="*.tsx"`
- [ ] Нет `@ts-ignore` / `@ts-expect-error`
- [ ] Shared code в `packages/` (не copy-paste между apps)
- [ ] Нет дублирования GraphQL queries между next-admin и next-frontend

### 19.10 Logging quality

- [x] Service methods имеют `#[instrument]` decorator
- [x] Нет логирования PII (email, password, token) — проверено при аудите
- [x] Нет `println!` / `eprintln!` в production коде — не найдено в crates/apps
- [ ] Structured fields вместо string formatting в tracing

### 19.11 Dependency antipatterns

- [x] `rustok-core` не зависит от domain crates (нет circular dependencies)
- [x] Domain crates не вызывают друг друга напрямую (через events)
- [ ] `rustok-test-utils` — только в `[dev-dependencies]`
- [ ] Нет `path` dependencies на crates вне workspace

### 19.12 API antipatterns — GraphQL

- [ ] Нет N+1 queries в resolvers (все связанные данные через DataLoader)
  - Искать: resolvers с прямыми DB-запросами внутри `async fn` для дочерних объектов
- [ ] `MergedObject` используется для модульной schema (не монолитный Query/Mutation)
- [ ] Нет `String` errors в GraphQL — используются structured error extensions
  - `grep -rn "FieldError::new" apps/server/src/graphql/ --include="*.rs"`
- [ ] Каждый mutation имеет permission check (не полагается на «auth достаточно»)
- [ ] Каждый query с list возвращает paginated результат (не полную таблицу)
- [ ] `context.data::<TenantContext>()` используется в каждом resolver (не пропущен)
- [ ] Нет бизнес-логики в resolvers — только вызов domain services
- [ ] Naming convention: queries — `camelCase`, mutations — `camelCase` с глаголом (`createProduct`, не `productCreate`)
- [ ] Subscription (если есть) использует WebSocket, не polling

### 19.13 API antipatterns — REST

- [ ] Каждый endpoint имеет `#[utoipa::path(...)]` annotation для OpenAPI
  - Искать: handlers без `#[utoipa::path]` в `apps/server/src/controllers/`
- [ ] HTTP status codes корректны:
  - 201 Created для POST (не 200)
  - 204 No Content для DELETE (не 200)
  - 404 Not Found для отсутствующих ресурсов (не 500)
  - 422 Unprocessable Entity для validation errors (не 400)
- [ ] Нет бизнес-логики в controllers — только вызов domain services
- [ ] `loco_rs::Result` для error handling (не custom error types)
- [ ] Все `CreateInput`/`UpdateInput` проходят через `validator::Validate`
- [ ] Нет endpoints без пагинации в list-запросах
- [ ] Rate limiting применён к auth endpoints (login, register, reset-password)
- [ ] CORS middleware подключён с правильными origins

### 19.14 REST ↔ GraphQL parity

- [ ] Auth операции (login, register, refresh, change-password) доступны и через REST, и через GraphQL
- [ ] Бизнес-логика **одна** — через `AuthLifecycleService` (не дублирована)
- [ ] RBAC проверки **идентичны** в REST и GraphQL для одной и той же операции
- [ ] Tenant isolation **идентичен** в обоих transport-слоях
- [ ] Error responses маппятся одинаково (один domain error → одинаковый HTTP status и GraphQL error code)
- [ ] Если CRUD операция доступна через REST — она доступна и через GraphQL (и наоборот, за исключением public storefront queries)

---

## Фаза 20: Правильность написания кода (Code Correctness)

### 20.1 Type Safety

- [ ] ID типы используют newtype pattern (`TenantId(Uuid)`) — не голые `Uuid`
- [ ] Status поля — enum, не `String`
- [ ] Phantom types для state-aware structs (если применимо)
- [ ] Нет `as` casts для числовых типов без проверки (use `TryFrom`)

### 20.2 Concurrency correctness

- [ ] `Arc<Mutex<>>` используется вместо `Mutex<>` для shared state между tasks
- [ ] `RwLock` для read-heavy shared state
- [ ] Нет race conditions в cache operations (singleflight pattern)
- [ ] `Select!` корректно обрабатывает cancellation

### 20.3 Resource management

- [ ] DB connections возвращаются в pool (нет leaked connections)
- [ ] Files закрываются (RAII через `Drop`)
- [ ] HTTP clients имеют timeouts
- [ ] Retry logic с exponential backoff (не busy-loop)

### 20.4 Serialization correctness

- [ ] `serde(rename_all = "camelCase")` для JSON APIs (если convention)
- [ ] `#[serde(skip_serializing)]` для sensitive fields
- [ ] DateTime сериализуется в ISO 8601
- [ ] UUID сериализуется как string (не binary)

### 20.5 Migration correctness

- [ ] Каждая миграция имеет `up()` и `down()` (reversible)
- [ ] Foreign keys с `ON DELETE CASCADE` или `ON DELETE SET NULL` (не orphans)
- [ ] Indexes на часто фильтруемые поля (`tenant_id`, `slug`, `status`)
- [ ] `UNIQUE` constraints где бизнес-логика требует уникальности

---

## Фаза 21: Реестр найденных проблем

Этот раздел — живой трекер всех `[!]` пунктов из других фаз.  
Добавляем сюда при обнаружении, обновляем при исправлении.

| № | Приоритет | Статус | Описание | Файлы | Фаза |
|---|-----------|--------|----------|-------|------|
| 1 | 🔴 Критический | ✅ Исправлено | `content` был помечен `required = true` в `modules.toml`, но `ContentModule::kind()` возвращает `ModuleKind::Optional`. Несоответствие приводило к ошибке `validate_registry_vs_manifest()` при старте. | `modules.toml` | 1.1 |
| 2 | 🔴 Критический | ⏳ Ожидает исправления | `rustok-blog` и `rustok-forum` используют `event_bus.publish()` вместо `publish_in_tx()` — нарушение атомарности, возможна потеря событий при сбое после commit DB-транзакции. | `crates/rustok-blog/src/services/post.rs`, `crates/rustok-forum/src/services/{topic,reply,moderation}.rs` | 6.2, 7.3, 7.4 |
| 3 | 🟡 Высокий | ✅ Исправлено | `iggy` версия `0.9.2` не существует на crates.io. CI-сборка падала. Исправлено на `0.9.0`. | `Cargo.toml`, `crates/rustok-iggy-connector/Cargo.toml` | 0.6 |

### 21.1 Детали: Проблема #2 — Небезопасная публикация событий в blog/forum

**Корневая причина:**  
`PostService` и `TopicService`/`ReplyService`/`ModerationService` принимают `TransactionalEventBus` и передают его в `NodeService` (который корректно использует `publish_in_tx()`). Но затем сами дополнительно вызывают `self.event_bus.publish()` для публикации модуль-специфичных событий (`BlogPostCreated`, `ForumTopicCreated`, etc.) — это происходит **вне транзакции**.

**Риск:**
1. `NodeService` выполняет операцию + `publish_in_tx()` в транзакции — всё атомарно.
2. `PostService.create_post()` вызывает `NodeService.create_node()` (успешно).
3. Затем вызывает `self.event_bus.publish(BlogPostCreated{...})` — это отдельная операция.
4. Если шаг 3 фейлится — основные данные уже в БД, но blog-специфичное событие потеряно.

**Рекомендуемое исправление:**
- Рефакторинг: вместо делегирования в NodeService с последующим отдельным publish — 
  использовать паттерн открытой транзакции: создать транзакцию в `PostService`, передать её в NodeService и в последующий `publish_in_tx()`.
- Или: убрать дублирующие события в blog/forum — NodeService уже публикует `NodeCreated`/`NodeUpdated`/etc., а IndexService может слушать их напрямую.

**Чеклист исправления:**
- [ ] Рефакторинг `PostService::create_post()` → `publish_in_tx()`
- [ ] Рефакторинг `PostService::update_post()` → `publish_in_tx()`
- [ ] Рефакторинг `PostService::publish_post()` → `publish_in_tx()`
- [ ] Рефакторинг `PostService::unpublish_post()` → `publish_in_tx()`
- [ ] Рефакторинг `PostService::delete_post()` → `publish_in_tx()`
- [ ] Рефакторинг `TopicService` → `publish_in_tx()`
- [ ] Рефакторинг `ReplyService::create_reply()` → `publish_in_tx()`
- [ ] Рефакторинг `ModerationService` (3 вызова) → `publish_in_tx()`
- [ ] Добавить integration тест: проверить что BlogPostCreated публикуется атомарно

---

## Итоговый отчёт

После прохождения всех фаз заполнить итоговую таблицу:

| Фаза | Название | Всего проверок | OK | Проблемы | Не применимо |
|------|---------|---------------|-----|----------|-------------|
| 0 | Компиляция и сборка | | | | |
| 1 | Соответствие архитектуре | | | | |
| 2 | Ядро платформы | | | | |
| 3 | Авторизация | | | | |
| 4 | RBAC | | | | |
| 5 | Multi-Tenancy | | | | |
| 6 | Событийная система | | | | |
| 7 | Доменные модули | | | | |
| 8 | API GraphQL | | | | |
| 9 | API REST | | | | |
| 10 | Фронтенды Leptos | | | | |
| 11 | Фронтенды Next.js | | | | |
| 12 | Фронтенд-библиотеки | | | | |
| 13 | Интеграционные связи | | | | |
| 14 | Тестовое покрытие | | | | |
| 15 | Observability | | | | |
| 16 | Документация | | | | |
| 17 | CI/CD | | | | |
| 18 | Безопасность | | | | |
| 19 | Антипаттерны и качество кода | | | | |
| 20 | Правильность написания кода | | | | |
| **ИТОГО** | | | | | |

---

## Связанные документы

- [Паттерны vs Антипаттерны](./standards/patterns-vs-antipatterns.md) — сводная таблица правильного и неправильного
- [Запрещённые действия](./standards/forbidden-actions.md) — что ни в коем случае нельзя делать
- [Known Pitfalls](./ai/KNOWN_PITFALLS.md) — ловушки для агентов
- [Architecture Overview](./architecture/overview.md)
- [API Architecture](./architecture/api.md)
- [RBAC Enforcement](./architecture/rbac.md)
- [Tenancy](./architecture/tenancy.md)
- [Events & Outbox](./architecture/events.md)
- [Improvement Recommendations](./architecture/improvement-recommendations.md)
- [Module Registry](./modules/registry.md)
- [Documentation Map](./index.md)
- [Testing Guidelines](./guides/testing.md)
- [Security Audit Guide](./guides/security-audit.md)

---

> **Как использовать этот план:** Открываем каждую фазу последовательно. При нахождении проблемы — фиксим в коде и ставим `[x]`. Если что-то не реализовано — помечаем `[~]` и создаём задачу. По завершению — собираем итоговый отчёт.
