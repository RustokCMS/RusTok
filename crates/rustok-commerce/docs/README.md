# rustok-commerce docs

В этой папке хранится документация модуля `crates/rustok-commerce`.

## Documents

- [Implementation plan](./implementation-plan.md) — план развития модуля

## Статус адаптеров

- GraphQL и REST адаптеры commerce теперь живут внутри `crates/rustok-commerce`
  (`src/graphql/*`, `src/controllers/*`).
- `apps/server` больше не содержит бизнес-логики commerce-адаптеров и использует только
  thin shim/re-export слой для маршрутов, OpenAPI и GraphQL schema composition.
- Общие transport-контракты (`AuthContext`, `TenantContext`, `RequestContext`,
  `require_module_enabled`, locale/pagination helper-ы) модуль получает из `rustok-api`.

## Event contracts

- [Event flow contract (central)](../../../docs/architecture/event-flow-contract.md)

