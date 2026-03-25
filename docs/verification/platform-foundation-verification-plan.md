# План верификации платформы: foundation

- **Статус:** Актуализированный детальный чеклист
- **Контур:** Сборка, workspace, архитектурная консистентность, ядро платформы, auth, RBAC, tenancy
- **Companion-план:** [Rolling-план RBAC для server и runtime-модулей](./rbac-server-modules-verification-plan.md)

---

## Фаза 0: Компиляция и сборка

### 0.1 Workspace и toolchain

- [ ] `cargo check --workspace` проходит в актуальной среде.
- [ ] `cargo check --workspace --all-features` проходит без feature-drift.
- [ ] `cargo clippy --workspace --all-features -- -D warnings` проходит или known-failures задокументированы отдельно.
- [ ] `cargo fmt --all -- --check` проходит.
- [ ] Для Leptos-части подтверждены Rust/WASM/Trunk prerequisites.
- [ ] Для Next.js-части подтверждены Node/npm prerequisites.

### 0.2 Основные пакеты сборки

- [ ] `cargo build -p rustok-server`
- [ ] `cargo build -p rustok-admin`
- [ ] `cargo build -p rustok-storefront`
- [ ] `cargo build -p xtask`
- [ ] `cargo build -p benches`

### 0.3 Ключевые crate-ы платформы

- [ ] `rustok-core`
- [ ] `rustok-auth`
- [ ] `rustok-cache`
- [ ] `rustok-email`
- [ ] `rustok-events`
- [ ] `rustok-outbox`
- [ ] `rustok-index`
- [ ] `rustok-tenant`
- [ ] `rustok-rbac`
- [ ] `rustok-content`
- [ ] `rustok-commerce`
- [ ] `rustok-blog`
- [ ] `rustok-forum`
- [ ] `rustok-pages`
- [ ] `rustok-media`
- [ ] `rustok-workflow`
- [ ] `rustok-api`
- [ ] `rustok-storage`
- [ ] `rustok-telemetry`
- [ ] `rustok-iggy`
- [ ] `rustok-iggy-connector`
- [ ] `rustok-mcp`
- [ ] `rustok-test-utils`
- [ ] `alloy`
- [ ] `alloy-scripting`
- [ ] `flex`

### 0.4 Frontend и UI workspace

- [ ] `apps/next-admin`: `npm run build`
- [ ] `apps/next-admin`: `npm run lint`
- [ ] `apps/next-frontend`: `npm run build`
- [ ] `apps/next-frontend`: `npm run lint`
- [ ] `apps/next-frontend`: `npm run typecheck`
- [ ] `UI/leptos`: cargo build проходит

### 0.5 Вспомогательные инструменты и окружение

- [ ] `make help` отражает актуальные targets.
- [ ] `docker compose config` для `docker-compose.yml` проходит.
- [ ] `docker compose -f docker-compose.yml -f docker-compose.full-dev.yml config` проходит.
- [ ] `docker compose -f docker-compose.observability.yml config` проходит.

---

## Фаза 1: Архитектурная консистентность

### 1.1 `modules.toml` и runtime registry

**Файлы:**
- `modules.toml`
- `apps/server/src/modules/mod.rs`
- `apps/server/src/modules/manifest.rs`

- [ ] Подтверждено, что `validate_registry_vs_manifest()` вызывается при старте сервера.
- [ ] Runtime-модули совпадают между manifest и `build_registry()`:
  - Core: `auth`, `cache`, `email`, `index`, `outbox`, `tenant`, `rbac`
  - Optional: `content`, `commerce`, `blog`, `forum`, `pages`, `media`, `workflow`
- [ ] Подтверждено, что `rustok-outbox` зарегистрирован как обычный `Core` module и одновременно используется сервером в event runtime bootstrap.
- [ ] `required = true` совпадает с `ModuleKind::Core` для platform modules.
- [ ] `depends_on` в manifest совпадает с `dependencies()` в `RusToKModule` impl.

### 1.2 Workspace members

- [ ] `Cargo.toml` workspace members покрывают `apps/server`, `apps/admin`, `apps/storefront`, `crates/*`, `UI/leptos`, `benches`, `xtask`.
- [ ] В workspace отражены crate-owned UI packages, которые действительно должны собираться вместе с workspace.
- [ ] Нет orphan-crate директорий с `Cargo.toml`, выпавших из workspace.

### 1.3 Текущая taxonomy компонентов

- [ ] Проверено, что shared library / support crate layer вне module taxonomy включает:
  - `rustok-core`, `rustok-events`, `rustok-telemetry`, `rustok-api`, `rustok-storage`, `rustok-test-utils`, `rustok-iggy`, `rustok-iggy-connector`, `rustok-mcp`, `alloy`, `alloy-scripting`, `flex`
- [ ] Проверено, что runtime core-модули с `ModuleKind::Core` сейчас: `auth`, `cache`, `email`, `index`, `outbox`, `tenant`, `rbac`.
- [ ] Проверено, что runtime optional-модули сейчас: `content`, `commerce`, `blog`, `forum`, `pages`, `media`, `workflow`.
- [ ] Проверено, что Alloy остаётся capability layer, а не tenant-toggle runtime module.

### 1.4 Зависимости между runtime-модулями

- [ ] `blog -> content`
- [ ] `forum -> content`
- [ ] `pages -> content`
- [ ] `media` не вводит скрытых runtime dependencies
- [ ] `workflow` не вводит скрытых runtime dependencies
- [ ] В registry-managed графе нет циклических зависимостей.

---

## Фаза 2: Ядро платформы

### 2.1 `rustok-core`

- [ ] `RusToKModule`, `ModuleKind`, `MigrationSource`, `ModuleRegistry` отражают текущий runtime contract.
- [ ] `Permission`, `Resource`, `Action` покрывают актуальные поверхности: content, commerce, blog, forum, pages, media, scripts, MCP, workflow, flex, users/modules/settings.
- [ ] `SecurityContext` и scope-модель соответствуют текущим server-side usage patterns.
- [ ] Cache abstractions (`InMemory`, `Redis`, fallback) совпадают с реальным использованием в server runtime.
- [ ] Error taxonomy и `ErrorResponse` совпадают с transport-layer mapping.

### 2.2 `rustok-events` и `rustok-outbox`

- [ ] `DomainEvent` / `EventEnvelope` остаются каноническим event contract.
- [ ] `TransactionalEventBus` и transport abstraction соответствуют текущему write-path.
- [ ] `OutboxRelay` и `sys_events` flow соответствуют текущему server runtime.
- [ ] `rustok-outbox` остаётся `Core` module и не участвует в tenant toggle flow как `Optional` module.

### 2.3 `rustok-telemetry`

- [ ] Telemetry config, tracing subscriber и OTEL wiring соответствуют текущему server bootstrap.
- [ ] Metrics API отражает реальные метрики, публикуемые сервером.
- [ ] Shutdown semantics актуальны для текущего `apps/server/src/main.rs`.

### 2.4 `rustok-api` и shared host contracts

- [ ] `rustok-api` остаётся тонким shared host/API layer для tenant/auth/request contracts.
- [ ] `TenantContext`, `AuthContext`, extension traits и error model используются сервером и host-приложениями консистентно.

---

## Фаза 3: Авторизация и аутентификация

### 3.1 Auth surfaces

**Файлы:**
- `crates/rustok-auth/`
- `apps/server/src/controllers/auth.rs`
- `apps/server/src/graphql/auth/`
- `apps/server/src/services/auth_lifecycle.rs`

- [ ] REST и GraphQL auth surfaces проходят через единый lifecycle/service layer.
- [ ] В коде покрыты: sign up, sign in, refresh, logout, password reset, change password, profile update, sessions.
- [ ] Session invalidation и password-change invalidation соответствуют текущей схеме таблиц/claims.
- [ ] Email verification и invite acceptance отражены как в коде, так и в плане.

### 3.2 JWT и session contract

- [ ] Claims содержат только реально используемые поля.
- [ ] Bearer extraction и session revocation работают через текущие extractors/services.
- [ ] Browser/API auth flows не расходятся по формату токенов.

### 3.3 Password and identity guarantees

- [ ] Password hashing и verification соответствуют текущему `rustok-core`/`rustok-auth` contract.
- [ ] User/tenant binding в auth flows соответствует текущей multi-tenant модели.

---

## Фаза 4: RBAC

### 4.1 Typed permission surface

**Файлы:**
- `crates/rustok-core/src/permissions.rs`
- `crates/rustok-core/src/rbac.rs`
- `apps/server/src/extractors/rbac.rs`

- [ ] Typed permissions покрывают текущие ресурсы: users, tenants, modules, settings, flex, products, orders, pages, nodes, media, blog/forum, scripts, MCP, workflows.
- [ ] В `extractors/rbac.rs` есть extractors для реально используемых REST surfaces.
- [ ] Flex, MCP, scripts/alloy и workflow используют typed permissions, а не локальные string aliases.

### 4.2 Server-side enforcement

- [ ] REST handlers не используют `CurrentUser` как замену permission checks.
- [ ] GraphQL mutations/queries используют `RbacService` или permission-aware guards.
- [ ] `infer_user_role_from_permissions()` не используется как замена авторизации.
- [ ] Нет hardcoded `UserRole::Admin` / `UserRole::SuperAdmin` в критичных authorization path.

### 4.3 Runtime-module permission ownership

- [ ] `permissions()` у runtime-модулей соответствует реальным server callsites.
- [ ] README runtime-модулей содержит актуальные `## Interactions` и permission surface.
- [ ] `pages`, `media`, `workflow` отражены в RBAC vocabulary и server enforcement.

---

## Фаза 5: Multi-tenancy и module lifecycle

### 5.1 Tenant resolution и context propagation

- [ ] `TenantContext` импортируется из `rustok-api` и используется сервером консистентно.
- [ ] Tenant resolution middleware/guards соответствуют текущему runtime.
- [ ] Hostname/header-based resolution не расходится с документацией.

### 5.2 Tenant cache

- [ ] Positive/negative cache, stampede protection и Redis invalidation соответствуют текущей реализации.
- [ ] Cache metrics и invalidation channels отражают текущий код.

### 5.3 Tenant data isolation

- [ ] Domain tables и services по-прежнему tenant-scoped там, где это требуется.
- [ ] `tenant_modules` отражает текущую runtime toggle-модель.
- [ ] Отключение core-модулей по-прежнему запрещено через `registry.is_core()`.

### 5.4 Module lifecycle

- [ ] `ModuleLifecycleService` согласован с `ModuleRegistry`, manifest и build pipeline.
- [ ] Enable/disable проверяет dependencies и dependents для runtime optional-модулей.
- [ ] Build/deployment manifest flow не расходится с tenant module lifecycle.
