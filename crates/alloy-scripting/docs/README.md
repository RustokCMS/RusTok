# alloy-scripting docs

Документация модуля `crates/alloy-scripting`.

## Содержание

- [Alloy Concept](../../../docs/alloy-concept.md) — стратегическое видение Alloy: Self-Evolving Integration Runtime
- [implementation-plan.md](./implementation-plan.md) — архитектура, компоненты, flow выполнения, future improvements

## Краткий обзор

`alloy-scripting` — скриптовый движок на базе Rhai для пользовательской автоматизации.

### Основные возможности

1. **Event hooks** — скрипты срабатывают на события сущностей (before_create, after_update, on_commit)
2. **Cron scheduler** — scheduled выполнение по расписанию
3. **API triggers** — скрипты как HTTP endpoints
4. **Manual execution** — ручной запуск через API

### Безопасность

- Resource limits (max_operations, timeout, call_depth)
- Auto-disable после 3 ошибок подряд
- Sandboxed execution (no FS/network access)

### Интеграция с платформой

`alloy-scripting` зарегистрирован в `ModuleRegistry` как опциональный модуль (`ModuleKind::Optional`) через `AlloyModule` в `apps/server/src/modules/alloy.rs`.

Это обеспечивает:
- Видимость состояния в `/health/modules`
- RBAC-контроль доступа к скриптам через ресурс `Scripts` (create/read/update/delete/list/manage)
- Управление миграциями через единый механизм реестра (`ScriptsMigration`)

Модуль также предоставляет:
- `ScriptableEntity` trait для интеграции с доменными сущностями
- `HookExecutor` для удобного вызова hooks из сервисов
- `ScriptOrchestrator` для координации выполнения

Рантайм (`AlloyState`) инициализируется в `apps/server/src/app.rs::after_routes()` — это session-level состояние для GraphQL.

См. [implementation-plan.md](./implementation-plan.md) для деталей.
