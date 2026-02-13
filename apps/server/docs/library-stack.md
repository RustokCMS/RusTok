# Server library stack (core dependencies)

Этот документ фиксирует **основные библиотеки backend-стека** в `apps/server` и их роль в RusTok.

> Цель: чтобы разработчики и AI-агенты не гадали по случайным статьям, а быстро видели «официальный» стек сервера в этом репозитории.

## Базовые библиотеки (сердце платформы)

| Библиотека | Роль в сервере | Где смотреть в репозитории |
|---|---|---|
| `loco-rs` | Backend framework, bootstrap app, env/config/runtime conventions | `apps/server/src/app.rs`, `apps/server/src/main.rs`, `apps/server/docs/loco/` |
| `axum` | HTTP routing, handlers, middleware integration | `apps/server/src/controllers/**`, `apps/server/src/middleware/**` |
| `sea-orm` | ORM, сущности, запросы, миграции | `apps/server/src/models/**`, `apps/server/migration/**` |
| `async-graphql` | GraphQL schema/query/mutation/resolvers | `apps/server/src/graphql/**` |
| `tokio` | Async runtime для I/O и фоновых задач | точка входа сервера и async services в `apps/server/src/**` |
| `serde` / `serde_json` | (De)serialization для API, конфигов и payload | DTO/response/request структуры в `apps/server/src/**` |
| `tracing` | Structured logging/telemetry hooks | `apps/server/src/**` и интеграции telemetry/crates |
| `utoipa` | OpenAPI/Swagger-описание REST API | `apps/server/src/controllers/swagger.rs` |

## Как проверять актуальность стека

1. Проверяйте объявленные зависимости:

```bash
sed -n '1,220p' apps/server/Cargo.toml
```

2. Если меняется основной server-стек (добавили/убрали ключевую библиотеку), обновляйте этот файл в том же PR.

3. Для Loco-specific контекста и freshness-политики используйте:

- `apps/server/docs/loco/README.md`
- `make docs-check-loco`
- `make docs-sync-loco`
- `apps/server/docs/upstream-libraries/README.md`
- `make docs-sync-server-libs` / `make docs-check-server-libs`

## Граница документа

- Это **корневой reference по основным библиотекам**, а не полный туториал.
- Узкоспециализированные детали (например, transport/events/observability) выносите в отдельные markdown-файлы внутри `apps/server/docs/`.
