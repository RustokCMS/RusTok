# admin docs

В этой папке хранится документация модуля `apps/admin`.

## Зафиксированный стек интеграции
- UI/state: `leptos`, `leptos_router`, `Resource`/actions.
- GraphQL transport: внутренний crate `crates/leptos-graphql` (тонкий слой).
- HTTP: `reqwest`.
- Typed GraphQL (опционально): `graphql-client` на уровне приложений.

Цель: использовать battle-tested библиотеки и минимальный внутренний glue-код.
