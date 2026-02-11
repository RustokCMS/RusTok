# RusToK: Документация vs Реализация — Сравнительный Анализ

> **Дата анализа:** 2026-02-06  
> **Последнее обновление:** 2026-02-11  
> **Цель:** Определить, что из предложений уже реализовано, а что остаётся TODO

---

## Резюме

| Категория | Реализовано | Частично | TODO |
|-----------|:-----------:|:--------:|:----:|
| Event System | ✅ | - | - |
| RBAC | ✅ | - | - |
| Module System | ✅ | - | - |
| Index (CQRS) | ✅ | - | - |
| Outbox Pattern | ✅ | - | - |
| Iggy Streaming | - | ✅ | - |
| Health Checks | ✅ | - | - |
| Flex Module | - | - | ✅ NEW |
| Migrations Convention | - | - | 1 item |

### Изменения 2026-02-11

- ✅ Синхронизирован checklist по закрытым critical задачам (event versioning, transactional publishing, test-utils, tenant cache).
- ✅ Уточнена документация `rustok-test-utils`: `setup_test_db()` создаёт in-memory DB без автоматических миграций; для схемы используется `setup_test_db_with_migrations::<M>()`.

### Изменения 2026-02-06

- ✅ Health endpoints: `/health/live`, `/health/ready`, `/health/modules`
- ✅ `fn permissions()` в `RusToKModule` trait
- ✅ Permissions в `CommerceModule` и `ContentModule`

---

## 1. Event System (L0 → L1 → L2)

### ✅ ПОЛНОСТЬЮ РЕАЛИЗОВАНО

| Компонент | Файл | Статус |
|-----------|------|--------|
| `EventTransport` trait | `rustok-core/events/transport.rs` | ✅ |
| `ReliabilityLevel` enum | L0: InMemory, L1: Outbox, L2: Streaming | ✅ |
| `MemoryTransport` (L0) | `rustok-core/events/memory.rs` | ✅ |
| `OutboxTransport` (L1) | `rustok-outbox/transport.rs` | ✅ |
| `IggyTransport` (L2) | `rustok-iggy/transport.rs` | ✅ (каркас) |
| `EventEnvelope` | Все поля: id, correlation_id, causation_id, trace_id, retry_count | ✅ |
| `DomainEvent` enum | 40+ событий для Content, Commerce, User, Index, Tenant | ✅ |
| `EventHandler` trait | С `handles()` и `handle()` | ✅ |
| `EventDispatcher` | С HandlerBuilder, DispatcherConfig | ✅ |

**Вывод:** Документация описывает то, что уже есть. Никаких изменений не требуется.

---

## 2. RBAC System

### ✅ ПОЛНОСТЬЮ РЕАЛИЗОВАНО

| Компонент | Файл | Статус |
|-----------|------|--------|
| `Permission` struct | `Resource + Action` | ✅ |
| `Resource` enum | 17 ресурсов (Users, Products, Orders, etc.) | ✅ |
| `Action` enum | Create, Read, Update, Delete, List, Export, Import, Manage | ✅ |
| `PermissionScope` | All, Own, None | ✅ |
| `SecurityContext` | role + user_id + get_scope() | ✅ |
| Role-based sets | SuperAdmin, Admin, Manager, Customer | ✅ |
| `Rbac::has_permission()` | С поддержкой Manage как wildcard | ✅ |

### ⚠️ PARTIAL: Missing `fn permissions()` in RusToKModule

Документация предлагала:

```rust
impl RusToKModule for CommerceModule {
    fn permissions(&self) -> &[Permission] { ... }
}
```

**Текущее состояние:** `RusToKModule` trait **не имеет** метода `permissions()`.

**TODO:** Добавить метод для динамической регистрации permissions по модулям.

---

## 3. Module System

### ✅ ПОЛНОСТЬЮ РЕАЛИЗОВАНО

| Компонент | Файл | Статус |
|-----------|------|--------|
| `RusToKModule` trait | slug, name, description, version, dependencies | ✅ |
| `MigrationSource` trait | fn migrations() | ✅ |
| `ModuleContext` | db, tenant_id, config | ✅ |
| `on_enable()` / `on_disable()` | Lifecycle hooks | ✅ |
| `event_listeners()` | Для регистрации handlers | ✅ |
| `health()` async | Возвращает HealthStatus | ✅ |
| `HealthStatus` enum | Healthy, Degraded, Unhealthy | ✅ |
| `ModuleRegistry` | HashMap-based регистрация | ✅ |

**Вывод:** Module system полностью соответствует документации.

---

## 4. Index Module (CQRS Read Model)

### ✅ ПОЛНОСТЬЮ РЕАЛИЗОВАНО

| Компонент | Файл | Статус |
|-----------|------|--------|
| `IndexModule` | impl RusToKModule | ✅ |
| Content indexer | `rustok-index/content/` | ✅ |
| Product indexer | `rustok-index/product/` | ✅ |
| `Indexer` trait | `rustok-index/traits.rs` | ✅ |
| `LocaleIndexer` trait | Multilingual support | ✅ |
| Search module | `rustok-index/search/` | ✅ |

**Вывод:** CQRS read model соответствует архитектуре.

---

## 5. Outbox Pattern

### ✅ ПОЛНОСТЬЮ РЕАЛИЗОВАНО

| Компонент | Файл | Статус |
|-----------|------|--------|
| `OutboxTransport` | `rustok-outbox/transport.rs` | ✅ |
| Entity (sys_events) | `rustok-outbox/entity.rs` | ✅ |
| Status enum | Pending, Dispatched | ✅ |
| Migration | `rustok-outbox/migration.rs` | ✅ |
| Relay worker | `rustok-outbox/relay.rs` | ✅ (stub) |

**Вывод:** Outbox полностью реализован.

---

## 6. Iggy Streaming (L2)

### ⚠️ PARTIAL: Каркас есть, логика частичная

| Компонент | Файл | Статус |
|-----------|------|--------|
| `IggyTransport` | `rustok-iggy/transport.rs` | ✅ |
| Config | `rustok-iggy/config.rs` | ✅ |
| Serialization | `rustok-iggy/serialization.rs` | ✅ |
| Producer | `rustok-iggy/producer.rs` | ✅ |
| Consumer | `rustok-iggy/consumer.rs` | ⚠️ stub |
| DLQ | `rustok-iggy/dlq.rs` | ⚠️ stub |
| Replay | `rustok-iggy/replay.rs` | ⚠️ stub |

**Вывод:** Базовая интеграция есть, продвинутые фичи (consumer groups, DLQ, replay) — stubs.

---

## 7. Health Checks

### ✅ РЕАЛИЗОВАНО: liveness/readiness/modules + агрегация

**Реализованные endpoints (`apps/server/src/controllers/health.rs`):**

- `/health` — базовая проверка процесса и версии
- `/health/live` — liveness probe
- `/health/ready` — расширенный readiness probe
- `/health/modules` — агрегированный статус модулей

**Readiness контракт (детализированный JSON):**

- Общий `status`: `ok | degraded | unhealthy`
- `checks`: системные зависимости (`database`, `cache_backend`, `event_transport`, `search_backend`)
- `modules`: health по зарегистрированным модулям
- Для каждой проверки: `name`, `kind`, `criticality`, `status`, `latency_ms`, `reason`
- `degraded_reasons`: причины деградации/недоступности

**Надёжность checks:**

- Timeout на каждую проверку (fail-fast)
- In-process circuit breaker (failure threshold + cooldown)
- Разделение на критичные/некритичные зависимости:
  - critical failure → `unhealthy`
  - non-critical failure → `degraded`

---

## 8. Scripting (Alloy)

### ✅ РЕАЛИЗОВАНО

| Компонент | Файл | Статус |
|-----------|------|--------|
| `ScriptingContext` | `rustok-core/scripting/mod.rs` | ✅ |
| Engine | Rhai via `alloy_scripting` | ✅ |
| Storage | SeaORM-based | ✅ |
| Scheduler | Background execution | ✅ |
| Orchestrator | Script management | ✅ |

**Вывод:** Scripting уже async-only через background scheduler, как рекомендовано.

---

## 9. Migrations Convention

### ⚠️ НЕ ФОРМАЛИЗОВАНО

Документация предлагала:

```
mYYYYMMDD_<module>_<nnn>_<description>.rs
```

**Текущее состояние:** Миграции есть, но convention не формализована.

**TODO:** Проверить существующие миграции и применить naming convention.

---

## 10. Flex Module

### ❌ НЕ СУЩЕСТВУЕТ — Новый концепт

Flex — это **новый концепт** из архитектурного обсуждения, который ещё не реализован.

**TODO:** Создать `rustok-flex` crate согласно спецификации в `docs/modules/flex.md`.

---

## Сводная Таблица TODO

| # | Область | Задача | Приоритет |
|---|---------|--------|-----------|
| 1 | RBAC | Добавить `fn permissions()` в `RusToKModule` | Medium |
| 2 | Health | Поддерживать readiness-контракт и обновлять критичность зависимостей при изменениях инфраструктуры | Medium |
| 3 | Migrations | Формализовать naming convention | Low |
| 4 | Iggy | Реализовать consumer, DLQ, replay | Low (Phase 2) |
| 5 | Flex | Создать модуль согласно спецификации | Low (Phase 3) |

---

## Рекомендации

1. **Обновить документацию** — убрать пункты, которые уже реализованы, или пометить их как ✅
2. **Health endpoints** — это quick win, можно добавить за час
3. **`fn permissions()`** — добавить в trait и реализовать в модулях
4. **Flex откладываем** — это Phase 3 по roadmap

---

## Файлы для обновления

| Документ | Что обновить |
|----------|--------------|
| `docs/ARCHITECTURE_GUIDE.md` | Пометить реализованные пункты как ✅ |
| `docs/ROADMAP.md` | Обновить статусы Phase 1 задач |
| `docs/MANIFEST_ADDENDUM.md` | Добавить отметки "уже реализовано" |

This is an alpha version and requires clarification. Be careful, there may be errors in the text. So that no one thinks that this is an immutable rule.
