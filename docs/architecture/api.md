# API Architecture

Политика использования API-стилей описана в [`docs/architecture/routing.md`](./routing.md).

## Краткое резюме

RusToK использует гибридный подход: GraphQL для UI-клиентов, REST для интеграций и служебных сценариев.

| API | Endpoint | Назначение |
|-----|----------|-----------|
| GraphQL | `/api/graphql` | Единый endpoint для admin и storefront UI |
| REST | `/api/v1/…` | Внешние интеграции, webhooks, batch jobs |
| OpenAPI/Swagger | `/swagger` | Документация REST API (генерируется через `utoipa`) |
| Health | `/api/health` | Статус сервиса и модулей |
| Metrics | `/metrics` | Prometheus метрики |

## GraphQL схема

GraphQL схема формируется из per-domain объектов через `MergedObject`:

- `CommerceQuery` / `CommerceMutation` — `rustok-commerce`
- `ContentQuery` / `ContentMutation` — `rustok-content`
- `BlogQuery` / `BlogMutation` — `rustok-blog`
- `ForumQuery` / `ForumMutation` — `rustok-forum`
- `AlloyQuery` / `AlloyMutation` — `alloy-scripting`

Точка сборки схемы: `apps/server/src/graphql/schema.rs`

## Связанные документы

- [Routing policy](./routing.md) — детальная policy GraphQL vs REST
- [Architecture overview](./overview.md)
- [UI GraphQL architecture](../UI/graphql-architecture.md)
