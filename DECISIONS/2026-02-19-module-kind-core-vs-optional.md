# Разделение модулей на Core и Optional

- Date: 2026-02-19
- Status: Accepted & Implemented

## Context

В текущей архитектуре `RusToKModule` trait не различает инфраструктурные модули (которые всегда должны быть активны) и доменные/опциональные модули (которые tenant может включать/отключать через `ModuleLifecycleService`).

Это приводит к нескольким проблемам:

1. `rustok-tenant`, `rustok-rbac`, `rustok-index` реализуют `RusToKModule`, но не зарегистрированы в `build_registry()` — их health-статус невидим, on_enable/on_disable хуки не вызываются.
2. `ModuleLifecycleService::toggle_module()` теоретически позволяет отключить `content`, от которого зависят `blog` и `forum`, если не заполнены `dependencies()`.
3. Нет machine-readable способа отличить что является ядром от того, что является опциональным расширением.

## Decision

Ввести поле `ModuleKind` в trait `RusToKModule`:

```rust
pub enum ModuleKind {
    Core,     // всегда активен, toggle запрещён
    Optional, // управляется per-tenant через ModuleLifecycleService
}

pub trait RusToKModule {
    fn kind(&self) -> ModuleKind {
        ModuleKind::Optional  // safe default
    }
}
```

Модули с `ModuleKind::Core` регистрируются в `ModuleRegistry` в отдельный `core_modules` bucket. `ModuleLifecycleService::toggle_module()` возвращает `ToggleModuleError::CoreModuleCannotBeDisabled` при попытке их отключения.

Следующие модули помечаются как Core:
- `IndexModule` (`rustok-index`) — CQRS read-path, критичен для storefront
- `TenantModule` (`rustok-tenant`) — tenant lifecycle хуки и health
- `RbacModule` (`rustok-rbac`) — RBAC lifecycle хуки и health

Следующие компоненты **не получают `ModuleKind`** — они не являются `RusToKModule`:
- `rustok-outbox` — инфраструктурный компонент, инициализируется через `build_event_runtime()`, а не через registry; является Compile-time Infrastructure
- `rustok-test-utils` — исключительно `[dev-dependencies]`, в production binary не входит
- `utoipa-swagger-ui-vendored` — vendored статика Swagger UI, не модуль платформы

Следующие модули остаются Optional:
- `ContentModule`, `CommerceModule`, `BlogModule`, `ForumModule`, `PagesModule`

`BlogModule` и `ForumModule` дополнительно заполняют `fn dependencies() -> &["content"]`.

## Consequences

**Положительные:**
- Явная граница между инфраструктурой и доменом.
- Health endpoint `/health/modules` начнёт отображать Tenant, RBAC, Index.
- `toggle_module()` станет безопасным: невозможно случайно отключить ядро.
- Документация и tooling могут автоматически строить граф зависимостей.

**Отрицательные:**
- Небольшой Breaking Change в trait `RusToKModule` — все реализации должны получить `fn kind()` (с default-значением Optional это non-breaking для existing modules).
- Требует обновления `ModuleRegistry` и `ModuleLifecycleService`.

**Follow-up:**
- Обновить `modules.toml` schema, добавив `required = true` для Core модулей.
- Обновить документацию в `docs/modules/overview.md`.
