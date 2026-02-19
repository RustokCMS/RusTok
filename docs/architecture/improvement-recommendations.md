# RusToK — Architecture Improvement Recommendations

- Date: 2026-02-19
- Status: Proposed
- Author: Platform Architecture Review

---

## 1. Контекст: что мы видим в коде сегодня

Прежде чем давать рекомендации, зафиксируем реальное состояние системы.

### 1.1 Граница между ядром и опциональными модулями

После анализа кода можно выделить **три категории** компонентов:

#### Категория A — Compile-time Infrastructure (не `RusToKModule`, не регистрируются)

Это «невидимые» для реестра crate'ы. Они линкуются в бинарник всегда, но не участвуют в lifecycle модулей:

| Crate | Роль | Почему не `RusToKModule` |
|---|---|---|
| `rustok-core` | Контракты, EventBus, RBAC, кэш, Circuit Breaker, метрики | Это само ядро, определяет trait |
| `rustok-iggy` + `rustok-iggy-connector` | L2 streaming transport (опциональный транспорт) | Технический адаптер, не бизнес-логика |
| `rustok-telemetry` | OpenTelemetry, tracing, Prometheus | Сквозная зависимость |
| `rustok-test-utils` | Фикстуры, моки, хелперы для тестов | **Только `dev-dependencies`**, в production binary не входит |
| `utoipa-swagger-ui-vendored` | Vendored Swagger UI assets | Статический ресурс, не модуль платформы |
| `alloy-scripting` | Скриптовый движок Rhai | Сейчас инициализируется напрямую в `app.rs` |
| `tailwind-rs/css/ast` | CSS tooling | Build-time инструментарий |
| `rustok-mcp` | MCP адаптер с binary target | Отдельный сервер, не часть основного runtime |

#### Категория B — Core Platform Modules (регистрируются как `ModuleKind::Core`, нельзя отключить)

Это модули, реализующие `RusToKModule` и **обязательные для работы платформы**:

| Crate | Роль | Текущий статус |
|---|---|---|
| `rustok-index` | CQRS read-model, индексатор для storefront | Реализует `IndexModule`, но **не зарегистрирован** |
| `rustok-tenant` | Tenant metadata, lifecycle хуки | Реализует `TenantModule`, но **не зарегистрирован** |
| `rustok-rbac` | RBAC helpers, lifecycle хуки | Реализует `RbacModule`, но **не зарегистрирован** |

> **`rustok-outbox` — особый случай.** Он не реализует `RusToKModule` и не входит в registry, но является **критическим инфраструктурным компонентом ядра**: `TransactionalEventBus` используется при каждой write-операции во всех domain-модулях. Инициализируется через `build_event_runtime()` в `app.rs`, а не через `ModuleRegistry`. Остановка outbox = потеря гарантий доставки событий для всей платформы.

#### Категория C — Optional Domain Modules (регистрируются как `ModuleKind::Optional`, per-tenant toggle)

| Crate | Тип | Depends on |
|---|---|---|
| `rustok-content` | Domain (фактически required) | `rustok-core` |
| `rustok-commerce` | Domain | `rustok-core` |
| `rustok-blog` | Wrapper | `rustok-content` |
| `rustok-forum` | Wrapper | `rustok-content` |
| `rustok-pages` | Domain | `rustok-core` |

**Ключевые наблюдения:**
- `rustok-index`, `rustok-tenant`, `rustok-rbac` — Категория B: реализуют `RusToKModule`, имеют lifecycle-хуки, но пока не зарегистрированы.
- `rustok-outbox` — ядро платформы, но **не через registry**: это `EventTransport`-слой, инициализируемый отдельно.
- `rustok-test-utils` — **исключительно `[dev-dependencies]`**, в production binary не входит никогда.
- `utoipa-swagger-ui-vendored` — vendored статика Swagger UI, не `RusToKModule`.

### 1.2 Реальное состояние событийной системы (три уровня)

Три транспортных уровня — это **не иерархия**, а **три независимых режима**, выбираемых через `settings.rustok.events.transport`:

```
build_event_runtime()  ←  вызывается в app.rs::after_routes()
         │
   ┌─────┴─────────────────────────────────────────┐
   │                    │                           │
   ▼                    ▼                           ▼
L0: MemoryTransport   L1: OutboxTransport      L2: IggyTransport
(tokio::broadcast)    (PostgreSQL sys_events)  (внешний Iggy-сервер)
  dev/MVP only          production default       highload / replay
                              │
                         OutboxRelay
                         (tokio::spawn loop)
                         batch=100, retry×5
                         backoff 1s → 60s
                         relay target: MemoryTransport  ← !
```

**Критические факты по коду:**

1. **L1 relay пишет обратно в `MemoryTransport`**, а не в Iggy. L1→L2 pipeline как связная цепочка **не реализован** — это три независимых режима.

2. **L2 (Iggy)** при выборе `transport = "iggy"` делает `connector.connect()` синхронно при старте сервера. Если Iggy-сервер недоступен → сервер **падает при старте** с ошибкой `BadRequest`.

3. **`replay()` в `IggyTransport`** — заглушка: проверяет `is_initialized()` и возвращает `Ok(())`. Replay событий не реализован.

4. **Текущий production-путь:** `outbox` (L1). `memory` — только dev. `iggy` — инфраструктурно готов, но функционально incomplete.

5. **`rustok-outbox` — самый критичный компонент платформы**: `TransactionalEventBus` вызывается при каждой write-операции во всех domain-модулях. Его остановка = полная остановка write-path.

### 1.3 Реализация кэша

Кэш **двухслойный** с автоматическим fallback и используется **только для tenant resolution**:

```
RUSTOK_REDIS_URL / REDIS_URL задан?
    ├── ДА  → RedisCacheBackend  (feature = "redis-cache")
    │            ├── CircuitBreaker на каждый GET/SET/DEL
    │            ├── При open circuit → Error::Cache, warn в лог
    │            └── Ключ: prefix + ":" + key
    └── НЕТ → InMemoryCacheBackend (moka)
                 TTL = global при инициализации
                 Capacity = 1000 записей
```

**Два отдельных кэша для каждого запроса:**

| Кэш | Ключ | TTL | Назначение |
|---|---|---|---|
| `tenant_cache` | `tenant:v1:{uuid\|slug\|host}:value` | 5 мин | Найденные tenants |
| `tenant_negative_cache` | `tenant_negative:v1:{uuid\|slug\|host}:value` | 60 сек | Несуществующие tenants (flood protection) |

**Stampede protection** через `in_flight: Arc<Mutex<HashMap<String, Arc<Notify>>>>`:
- 100 параллельных запросов по одному tenant → только **1** идёт в БД, остальные ждут `Notify::notified()`.

**Cross-instance invalidation** (только при Redis):
- При обновлении tenant → `PUBLISH tenant.cache.invalidate <key>|<neg_key>` в Redis pub/sub.
- Все инстансы подписаны и локально инвалидируют оба ключа.
- Метрики (hits/misses) тоже пишутся в Redis через `INCR` → `/metrics` показывает агрегат кластера.

**Известная проблема:** `InMemoryCacheBackend::set_with_ttl()` **игнорирует параметр `_ttl`**, используя только глобальный TTL, заданный при создании кэша. Per-entry TTL не работает в in-memory режиме.

---

## 2. Рекомендации

### 2.1 ✅ РЕАЛИЗОВАНО: Устранить размытую границу core / domain-module

**Проблема.** `rustok-tenant`, `rustok-rbac` и `rustok-index` имеют `impl RusToKModule`, но не регистрировались в `build_registry()`. Не было формального способа отличить обязательные модули от опциональных.

> **Что точно является Core (нельзя отключить):** `rustok-index`, `rustok-tenant`, `rustok-rbac`.
>
> **Что точно НЕ является частью `ModuleKind` системы:**
> - `rustok-test-utils` — это `[dev-dependencies]`, в production binary не входит вообще.
> - `utoipa-swagger-ui-vendored` — vendored статика Swagger UI, не `RusToKModule`.
> - `rustok-outbox` — не реализует `RusToKModule`; это инфраструктурный компонент, инициализируемый через `build_event_runtime()`, а не через registry.

**Рекомендация.** Ввести формальную классификацию на уровне trait'а:

```rust
// В rustok-core/src/module.rs
pub enum ModuleKind {
    /// Всегда активен, не может быть отключён tenant'ом
    Core,
    /// Регистрируется в реестре, может быть включён/отключён per-tenant
    Optional,
}

pub trait RusToKModule: Send + Sync + MigrationSource {
    fn kind(&self) -> ModuleKind {
        ModuleKind::Optional  // safe default
    }
    // ... остальное без изменений
}
```

**Решение реализовано:**
- В `rustok-core/src/module.rs` добавлен `ModuleKind { Core, Optional }` с doc-комментарием-предупреждением.
- `fn kind(&self) -> ModuleKind` добавлен в трейт `RusToKModule` с default `Optional`.
- `ModuleRegistry` разделён на два bucket'а: `core_modules` и `optional_modules`.
- `ModuleLifecycleService::toggle_module()` возвращает `ToggleModuleError::CoreModuleCannotBeDisabled` при попытке отключить core-модуль.
- `IndexModule`, `TenantModule`, `RbacModule` переопределяют `fn kind() -> ModuleKind::Core`.
- Все три зарегистрированы в `build_registry()` первыми.

---

### 2.2 ✅ РЕАЛИЗОВАНО: Зарегистрировать `rustok-index` в `build_registry()`

**Проблема.** `rustok-index` не был зарегистрирован. CQRS read-path не работал.

**Решение:** `IndexModule` добавлен в `apps/server/src/modules/mod.rs` как первый Core-модуль.

---

### 2.3 ✅ РЕАЛИЗОВАНО: Зарегистрировать `TenantModule` и `RbacModule`

**Проблема.** `TenantModule` и `RbacModule` реализовали `RusToKModule` с `health()`, но не были зарегистрированы — их health-статус отсутствовал в `/health/modules`.

**Решение:** Оба зарегистрированы в `build_registry()` как Core-модули. Теперь их health виден, хуки вызываются, миграции подхватываются автоматически.

---

### 2.4 🟡 ВАЖНО: Согласовать `modules.toml` с реальностью `build_registry()`

**Проблема.** `modules.toml` — декларативный манифест, но `build_registry()` — императивный Rust-код. Сегодня они могут расходиться: в `modules.toml` есть `index` и `outbox`, но в `build_registry()` их нет.

**Рекомендация.** Либо:
1. **Генерировать `build_registry()` из `modules.toml`** (через xtask/build.rs) — единый source of truth.
2. **Добавить проверку на старте** — сравнивать содержимое `modules.toml` с реально зарегистрированными slug'ами и завершать процесс с ошибкой при расхождении.

Второй вариант быстрее реализовать:

```rust
// В app.rs, после build_registry()
let registry = modules::build_registry();
modules::validate_registry_vs_manifest(&registry)
    .expect("modules.toml is out of sync with build_registry()");
```

---

### 2.5 🟡 ВАЖНО: Добавить `is_required` / `dependencies` для Content-модуля

**Проблема.** `rustok-blog` и `rustok-forum` — это wrapper-модули поверх `rustok-content`. Если `content` отключить через `toggle_module()`, `blog` и `forum` сломаются без явной ошибки. Сейчас `dependencies()` в `BlogModule` и `ForumModule` возвращает `&[]`.

**Рекомендация.** Заполнить зависимости:

```rust
// В BlogModule
fn dependencies(&self) -> &[&'static str] {
    &["content"]
}

// В ForumModule
fn dependencies(&self) -> &[&'static str] {
    &["content"]
}
```

`ModuleLifecycleService` уже проверяет зависимости при `toggle_module(enabled=true)` и наличие зависимых при `toggle_module(enabled=false)`. Достаточно только заполнить поле.

---

### 2.6 🟡 ВАЖНО: Ввести `required` флаг в `modules.toml`

**Проблема.** В `modules.toml` нет способа пометить модуль как неотключаемый. Комментарий `# Core modules (required)` — это просто комментарий, а не машиночитаемое ограничение.

**Рекомендация.** Расширить схему манифеста:

```toml
[modules]
content = { crate = "rustok-content", source = "path", path = "crates/rustok-content", required = true }
index   = { crate = "rustok-index",   source = "path", path = "crates/rustok-index",   required = true }

commerce = { crate = "rustok-commerce", source = "path", path = "crates/rustok-commerce" }
blog     = { crate = "rustok-blog",     source = "path", path = "crates/rustok-blog",     depends_on = ["content"] }
forum    = { crate = "rustok-forum",    source = "path", path = "crates/rustok-forum",     depends_on = ["content"] }
pages    = { crate = "rustok-pages",    source = "path", path = "crates/rustok-pages" }
```

Это позволит xtask/CLI инструментам и документации автоматически знать, что можно включать/отключать.

---

### 2.7 🟡 ВАЖНО: Связать L1 (Outbox) и L2 (Iggy) в единый pipeline

**Проблема.** Сейчас три транспортных уровня — это три **независимых режима**, а не иерархия. `OutboxRelay` пишет события в `MemoryTransport` (не в Iggy). При выборе `transport = "iggy"` outbox-гарантии теряются. Нет плавного пути L1 → L2.

**Рекомендация.** Сделать `OutboxRelay` настраиваемым: relay target должен поддерживать `IggyTransport` как downstream:

```toml
[settings.rustok.events]
transport = "outbox"           # write-side: всегда outbox
relay_target = "iggy"         # read-side: relay в Iggy для replay и highload
```

Это обеспечит: AtLeastOnce через outbox + replay через Iggy + не падать при старте если Iggy временно недоступен (lazy connect).

---

### 2.8 🟡 ВАЖНО: Исправить `InMemoryCacheBackend::set_with_ttl()`

**Проблема.** Метод `set_with_ttl()` в `InMemoryCacheBackend` игнорирует параметр `_ttl`:

```rust
async fn set_with_ttl(&self, key: String, value: Vec<u8>, _ttl: Duration) -> Result<()> {
    // Moka cache uses global TTL policy by default.
    // For now we just insert, ignoring specific TTL.  ← BUG
    self.cache.insert(key, value).await;
    Ok(())
}
```

Moka поддерживает per-entry TTL через `Cache::builder().time_to_idle()` и `insert_with_ttl()`. Сейчас отрицательный кэш (TTL 60s) и позитивный кэш (TTL 300s) оба используют один и тот же глобальный TTL того экземпляра, в который вставляется запись. Поведение соответствует ожидаемому только потому, что для каждого TTL создаётся **отдельный** `InMemoryCacheBackend`. Но `set_with_ttl` как API — обманчив.

**Рекомендация.** Реализовать per-entry TTL через `moka::future::Cache` с `time_to_live_per_entry`:

```rust
async fn set_with_ttl(&self, key: String, value: Vec<u8>, ttl: Duration) -> Result<()> {
    self.cache.insert_with_ttl(key, value, ttl).await;
    Ok(())
}
```

---

### 2.9 🟢 УЛУЧШЕНИЕ: Вынести `DomainEvent` из `rustok-core` в `rustok-events`

**Проблема.** `rustok-core/src/events/types.rs` содержит 37 556 байт — это огромный enum с событиями для content, blog, forum, commerce, user, pages, tenants, MCP. При добавлении нового модуля разработчик вынужден редактировать core, что нарушает принцип Open/Closed.

```rust
// сегодня: всё в одном месте
pub enum DomainEvent {
    NodeCreated { ... },      // content
    ProductCreated { ... },   // commerce
    OrderPlaced { ... },      // commerce
    PostPublished { ... },    // blog
    TopicCreated { ... },     // forum
    PagePublished { ... },    // pages
    // ...50+ вариантов
}
```

**Рекомендация.** Ввести паттерн расширяемых событий через типизированные "пространства":

```rust
// В rustok-core: только инфраструктурные события + envelope
pub enum CoreEvent {
    UserRegistered { user_id: Uuid, email: String },
    UserLoggedIn { user_id: Uuid },
    TenantCreated { tenant_id: Uuid },
}

// Каждый модуль объявляет свои события сам:
// rustok-commerce/src/events.rs
pub enum CommerceEvent {
    ProductCreated { product_id: Uuid },
    OrderPlaced { order_id: Uuid, tenant_id: Uuid },
}

// EventEnvelope параметризуется по payload:
pub struct EventEnvelope<E = serde_json::Value> {
    pub event_type: String,
    pub tenant_id: Uuid,
    pub event: E,
    // ...
}
```

Это Breaking Change, требует ADR. Приоритет: реализовать до появления первого production tenant.

---

### 2.8 🟢 УЛУЧШЕНИЕ: Добавить per-tenant module settings

**Проблема.** `tenant_modules.settings` (JSONB) уже существует в модели, но нигде не используется. `ModuleContext.config` принимает `&Value`, но сервисы его игнорируют.

**Рекомендация.** Ввести типизированный конфиг для модулей:

```rust
pub trait RusToKModule {
    type Config: DeserializeOwned + Default;

    fn parse_config(&self, raw: &Value) -> Result<Self::Config> {
        serde_json::from_value(raw.clone()).map_err(|e| Error::config(e))
    }
}
```

Это откроет возможность per-tenant настройки: лимиты постов для блога, список валют для commerce, кастомные SEO-поля и т.д.

---

### 2.9 🟢 УЛУЧШЕНИЕ: Добавить `rustok-notifications` как опциональный инфраструктурный модуль

**Проблема.** В RBAC и Events нет общего механизма уведомлений. Сейчас если нужно отправить email при `OrderPlaced` — это пишется в ad-hoc обработчиках.

**Рекомендация.** Создать `crates/rustok-notifications`:
- Подписывается на `DomainEvent` через EventBus.
- Поддерживает каналы: email (уже есть `apps/server/src/services/email.rs`), webhook, in-app.
- Конфигурируется per-tenant через `tenant_modules.settings`.
- Является опциональным Optional-модулем (не Core).

---

### 2.10 🟢 УЛУЧШЕНИЕ: Улучшить Outbox: добавить DLQ и мониторинг backlog

**Проблема.** `OutboxRelay::process_pending_once()` логирует ошибки но не отправляет застрявшие события в Dead Letter Queue. Нет метрик по backlog size.

**Рекомендация.**
1. Добавить `outbox_backlog_size` gauge в Prometheus metrics (экспортировать через `/metrics`).
2. Реализовать DLQ: после N попыток (конфигурируемое) переносить события в таблицу `sys_events_dlq`.
3. Добавить API endpoint `GET /api/admin/events/dlq` для просмотра и replay застрявших событий.

Базовый скелет DLQ уже есть в `crates/rustok-iggy/src/dlq.rs` — можно портировать логику в outbox-уровень.

---

### 2.11 🟢 УЛУЧШЕНИЕ: Формализовать Alloy Scripting как опциональный модуль

**Проблема.** `alloy-scripting` подключается напрямую в `app.rs` через `alloy_scripting::create_default_engine()` — минуя ModuleRegistry. Это нарушает единообразие модульной архитектуры.

**Рекомендация.** Обернуть в `RusToKModule`:

```rust
pub struct AlloyModule;

impl RusToKModule for AlloyModule {
    fn slug(&self) -> &'static str { "alloy" }
    fn kind(&self) -> ModuleKind { ModuleKind::Optional }
}
```

Это позволит:
- Включать/отключать скриптовый движок per-tenant.
- Отображать состояние в `/health/modules`.
- Ограничивать доступ через RBAC (`scripting:execute` permission).

---

### 2.12 🔵 СТРАТЕГИЧЕСКИ: Разделить `apps/server` на `core-server` + `module-bundles`

**Проблема.** Сейчас в `apps/server/src/app.rs` все маршруты (`commerce`, `blog`, `forum`, `content`, `pages`) захардкожены. При добавлении нового модуля нужно менять `app.rs`. Это создаёт coupling.

**Рекомендация.** Ввести автоматическую регистрацию роутов через trait:

```rust
pub trait HttpModule: RusToKModule {
    fn routes(&self) -> axum::Router;
}

// В app.rs:
fn routes(ctx: &AppContext) -> AppRoutes {
    let registry = modules::build_registry();
    let mut routes = AppRoutes::with_default_routes()
        .add_route(controllers::health::routes())
        .add_route(controllers::auth::routes())
        .add_route(controllers::graphql::routes());

    for module in registry.http_modules() {
        routes = routes.add_route(module.routes());
    }
    routes
}
```

Это позволит добавлять новые модули без изменения `app.rs`.

---

## 3. Приоритизированный план действий

| # | Рекомендация | Приоритет | Сложность | Блокирует |
|---|---|---|---|---|
| 2.1 | Ввести `ModuleKind::Core` / `Optional` | ✅ Готово | — | — |
| 2.2 | Зарегистрировать `rustok-index` | ✅ Готово | — | — |
| 2.3 | Зарегистрировать Tenant/RBAC как Core | ✅ Готово | — | — |
| 2.5 | Заполнить `dependencies()` для Blog/Forum | 🔴 Критично | Низкая | Data integrity |
| 2.4 | Синхронизация `modules.toml` ↔ `build_registry()` | 🟡 Важно | Средняя | Ops reliability |
| 2.6 | `required` / `depends_on` в `modules.toml` | 🟡 Важно | Низкая | Ops tooling |
| 2.7 | Связать L1 (Outbox) → L2 (Iggy) pipeline | 🟡 Важно | Высокая | Event highload |
| 2.8 | Исправить `set_with_ttl()` в InMemoryCache | 🟡 Важно | Низкая | Cache correctness |
| 2.12 | Outbox DLQ + backlog metrics | 🟢 Улучшение | Средняя | Event reliability |
| 2.10 | Per-tenant typed module config | 🟢 Улучшение | Средняя | Extensibility |
| 2.13 | Alloy как `RusToKModule` | 🟢 Улучшение | Низкая | Consistency |
| 2.11 | `rustok-notifications` модуль | 🟢 Улучшение | Высокая | New capability |
| 2.9 | Вынести `DomainEvent` из core | 🔵 Стратегически | Высокая | Extensibility |
| 2.14 | Авторегистрация HTTP routes | 🔵 Стратегически | Высокая | DX / scalability |

---

## 4. Итоговая картина после применения рекомендаций

```mermaid
graph TD
    subgraph "Kernel (always on)"
        CORE[rustok-core]
        OUTBOX[rustok-outbox + DLQ]
        INDEX[rustok-index ← registered]
        TENANT[rustok-tenant ← registered as Core]
        RBAC[rustok-rbac ← registered as Core]
        TELEMETRY[rustok-telemetry]
    end

    subgraph "Domain Modules (per-tenant toggle)"
        CONTENT[rustok-content]
        COMMERCE[rustok-commerce]
        BLOG[rustok-blog → depends: content]
        FORUM[rustok-forum → depends: content]
        PAGES[rustok-pages]
        NOTIF[rustok-notifications ← new]
    end

    subgraph "Optional Infrastructure"
        ALLOY[alloy-scripting ← as RusToKModule]
        MCP[rustok-mcp]
        IGGY[rustok-iggy]
    end

    CORE --> TENANT
    CORE --> RBAC
    CORE --> INDEX
    CONTENT --> BLOG
    CONTENT --> FORUM
    OUTBOX --> INDEX
    OUTBOX --> NOTIF
```

---

## 5. Связанные документы

- [`docs/architecture/overview.md`](./overview.md) — архитектурный обзор
- [`docs/architecture/principles.md`](./principles.md) — принципы архитектуры
- [`docs/architecture/events.md`](./events.md) — транзакционная публикация событий
- [`docs/modules/registry.md`](../modules/registry.md) — реестр компонентов
- [`docs/modules/overview.md`](../modules/overview.md) — состояние модулей
- [`DECISIONS/`](../../DECISIONS/) — архитектурные решения (ADR)
