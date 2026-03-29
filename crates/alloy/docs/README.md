# alloy docs

Документация capability-crate `crates/alloy`.

## Содержание

- [Alloy Concept](../../../docs/alloy-concept.md) — стратегическое видение Alloy
- [implementation-plan.md](./implementation-plan.md) — архитектура runtime, execution flow и дальнейшие шаги

## Краткий обзор

`alloy` — единый runtime/capability crate для Alloy на базе Rhai.

Он держит:

- storage, execution log и migrations для скриптов;
- `ScriptEngine`, `ScriptOrchestrator`, `Scheduler` и bridge/helper слой;
- GraphQL/HTTP transport surfaces (`graphql::*`, `controllers::routes`);
- интеграционные контракты `ScriptableEntity` / `HookExecutor` для host-модулей.

Публичный модуль Alloy не входит в tenant module registry и не требует `tenant_modules.is_enabled("alloy")`.
Transport-адаптеры GraphQL/HTTP живут в самом `alloy`, а `apps/server` подключает их через generated module wiring из `modules.toml` и `rustok-module.toml`.
