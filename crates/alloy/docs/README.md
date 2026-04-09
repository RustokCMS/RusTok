# Документация `alloy`

`alloy` — capability-модуль платформенного script/runtime слоя на базе Rhai.
Он входит в `ModuleRegistry` и устанавливается/удаляется как остальные optional
модули, но при этом остаётся capability-only слоем, а не tenant-бизнес-доменом.

## Назначение

- публиковать канонический runtime entry point для script execution;
- держать storage, execution log, scheduler и bridge/helper слой внутри capability crate;
- предоставлять единый contract для host integration без размазывания script runtime по `apps/server`.

## Зона ответственности

- `ScriptEngine`, `ScriptOrchestrator`, `Scheduler` и execution lifecycle;
- storage/migrations для scripts и execution log;
- GraphQL/HTTP transport surfaces (`graphql::*`, `controllers::routes`);
- интеграционные контракты `ScriptableEntity` и `HookExecutor` для host-модулей;
- отсутствие превращения script runtime в отдельный tenant-бизнес-домен.

## Интеграция

- подключается `apps/server` через generated module wiring из `modules.toml` и `rustok-module.toml`;
- регистрируется в `ModuleRegistry` как обычный optional модуль и публикует script permission surface;
- использует Rhai как embedded engine и должен удерживать sandbox/resource-limit semantics;
- может вызываться доменными модулями через hook/integration contracts, не размывая их собственные runtime boundaries.

## Проверка

- `cargo xtask module validate alloy`
- `cargo xtask module test alloy`
- targeted runtime tests для script execution, scheduler и bridge semantics при изменении capability surface

## Связанные документы

- [README crate](../README.md)
- [План реализации](./implementation-plan.md)
- [Alloy Concept](../../../docs/alloy-concept.md)
- [Контракт manifest-слоя](../../../docs/modules/manifest.md)
