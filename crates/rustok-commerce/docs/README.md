# Документация `rustok-commerce`

В этой папке хранится документация модуля `crates/rustok-commerce`.

## Документы

- [План реализации](./implementation-plan.md) — подробный план миграции `rustok-commerce` на Medusa-подобную архитектуру, backlog противоречий и ссылки на актуальные Medusa v2 API.
- [Пакет админского UI](../admin/README.md)
- [Пакет storefront UI](../storefront/README.md)

## Статус распила

- `rustok-cart`, `rustok-customer`, `rustok-product`, `rustok-pricing`, `rustok-inventory`, `rustok-order`, `rustok-payment` и `rustok-fulfillment` уже выделены в отдельные crates и platform modules.
- `rustok-commerce` теперь играет роль `Ecommerce` umbrella/root module для всего ecommerce family и держит transport/API surface, orchestration и legacy-части, которые ещё не вынесены в отдельные модули.
- Общие DTO, entities, error surface и search helpers вынесены в `rustok-commerce-foundation`.

## Статус адаптеров

- GraphQL и REST адаптеры commerce теперь живут внутри `crates/rustok-commerce` (`src/graphql/*`, `src/controllers/*`).
- `apps/server` больше не содержит бизнес-логики commerce-адаптеров и использует только thin shim/re-export слой для маршрутов, OpenAPI и GraphQL schema composition.
- Общие transport-контракты (`AuthContext`, `TenantContext`, `RequestContext`, `require_module_enabled`, locale/pagination helpers) модуль получает из `rustok-api`.
- Publishable Leptos admin UI для commerce теперь живёт в `crates/rustok-commerce/admin/`; host admin подключает пакет через manifest-driven `build.rs` и рендерит module-owned catalog control room на `/modules/commerce`.
- Publishable Leptos storefront UI для commerce теперь живёт в `crates/rustok-commerce/storefront/`; host storefront подключает пакет через manifest-driven `build.rs`, а public GraphQL read-path отдаёт published product catalog и selected product detail для `/modules/commerce`.

## Контракты событий

- [Event flow contract (central)](../../../docs/architecture/event-flow-contract.md)
