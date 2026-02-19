# Module Architecture

RusToK реализован как **Modular Monolith**: все модули компилируются в единый бинарник и поднимаются через `ModuleRegistry`.

## Ключевой принцип

Не каждый критичный компонент платформы реализует `RusToKModule`.
Есть три категории компонентов — подробно описаны ниже.

## Категория A — Compile-time Infrastructure (не `RusToKModule`)

Всегда линкуются в бинарник, не участвуют в lifecycle модулей:

| Crate | Роль |
|-------|------|
| `rustok-core` | Контракты, EventBus, кэш, Circuit Breaker — само ядро платформы |
| `rustok-outbox` | `TransactionalEventBus` для надёжной доставки событий |
| `rustok-iggy` + `rustok-iggy-connector` | L2 streaming transport (опционально) |
| `rustok-telemetry` | OpenTelemetry, tracing, Prometheus — сквозная зависимость |
| `alloy-scripting` | Скриптовый движок, инициализируется напрямую в `app.rs` |
| `rustok-mcp` | MCP адаптер, отдельный сервер |
| `tailwind-rs/css/ast` | Build-time CSS инструментарий |
| `rustok-test-utils` | **Только `[dev-dependencies]`**, никогда не попадает в production |

## Категория B — Core Platform Modules (`ModuleKind::Core`)

Реализуют `RusToKModule`, обязательны для работы платформы, нельзя отключить:

| Crate | Slug | Назначение |
|-------|------|-----------|
| `rustok-index` | `index` | CQRS read-model, индексатор для storefront |
| `rustok-tenant` | `tenant` | Tenant metadata, lifecycle hooks |
| `rustok-rbac` | `rbac` | RBAC helpers, lifecycle hooks |

## Категория C — Optional Domain Modules (`ModuleKind::Optional`)

Реализуют `RusToKModule`, управляются per-tenant через `tenant_modules`:

| Crate | Slug | Тип | Depends on |
|-------|------|-----|-----------|
| `rustok-content` | `content` | Domain (фактически required) | `rustok-core` |
| `rustok-commerce` | `commerce` | Domain | `rustok-core` |
| `rustok-blog` | `blog` | Wrapper | `rustok-content` |
| `rustok-forum` | `forum` | Wrapper | `rustok-content` |
| `rustok-pages` | `pages` | Domain | `rustok-core` |

**Wrapper-паттерн:** `rustok-blog` и `rustok-forum` не имеют собственных таблиц контента — они используют nodes из `rustok-content` с разными значениями поля `kind`.

## Где смотреть в коде

| Что | Где |
|-----|-----|
| Runtime-регистрация модулей | `apps/server/src/modules/mod.rs` |
| Синхронизация манифеста и registry | `apps/server/src/modules/manifest.rs` |
| Контракт модуля `RusToKModule` | `crates/rustok-core/src/module.rs` |
| Реестр Core/Optional | `crates/rustok-core/src/registry.rs` |
| Конфигурация состава модулей | `modules.toml` |

## Жизненный цикл модуля

```text
modules.toml → cargo build → ModuleRegistry::register() → on_enable() → runtime
```

Изменение состава модулей = изменение `modules.toml` + пересборка бинарника.
Runtime hot-plug отсутствует намеренно (compile-time safety).

## Связанные документы

- [Module overview](../modules/overview.md) — что зарегистрировано в сервере
- [Module & application registry](../modules/registry.md) — полный реестр компонентов
- [Module manifest](../modules/manifest.md) — формат `modules.toml` и rebuild lifecycle
- [Architecture overview](./overview.md)
- [Events and outbox](./events.md)
