# Server docs

В этой папке хранится документация модуля `apps/server`.

## Документы

- [`library-stack.md`](./library-stack.md) — основные backend-библиотеки сервера и их роль (framework, HTTP, ORM, GraphQL, runtime, observability).
- [`event-transport.md`](./event-transport.md) — как работает конфигурация и runtime-пайплайн транспорта событий.
- [`health.md`](./health.md) — health/readiness endpoints, модель агрегации статуса и circuit breaker.
- [`LOCO_FEATURE_SUPPORT.md`](./LOCO_FEATURE_SUPPORT.md) — decision matrix по Loco-функционалу vs самопису (anti-duplication baseline), включая статус Mailer/Workers/Storage и текущее состояние кэширования.
- [`CACHE_STAMPEDE_PROTECTION.md`](./CACHE_STAMPEDE_PROTECTION.md) — stampede protection в tenant cache: request coalescing, negative cache, Redis pub/sub invalidation.
- [`loco/README.md`](./loco/README.md) — Loco-specific контекст, workflow для агентов и freshness-политика upstream snapshot.
- [`upstream-libraries/README.md`](./upstream-libraries/README.md) — локальный snapshot актуальной внешней документации по ключевым crate сервера.

## Ключевые детали реализации

### Lifecycle hooks (Loco)

`impl Hooks for App` (`src/app.rs`) реализует весь lifecycle:

- `boot` — `create_app::<Self, Migrator>` запускает Loco bootstrap.
- `routes` — регистрирует REST/GraphQL контроллеры через `AppRoutes`.
- `after_routes` — строит event runtime, регистрирует tenant cache, scripting-контекст, tenant middleware. Сохраняет `Arc<EventRuntime>` в `shared_store` для `connect_workers`.
- `truncate` — удаляет таблицы в правильном порядке (соблюдает FK-ограничения).
- `connect_workers` — читает `Arc<EventRuntime>` из `shared_store` (построенный в `after_routes`) и запускает outbox relay worker, если транспорт `outbox`.
- `register_tasks` — регистрирует `cleanup` task.
- `initializers` — подключает `TelemetryInitializer`.
- `seed` — запускает `seed_development` / `seed_test` / `seed_minimal` по имени файла.

### Cleanup task

```bash
cargo loco task --name cleanup --args "sessions"
```

Удаляет просроченные сессии. Поддерживает targets: `sessions`, `cache`, пустая строка (full).

### Dev seed

`seed_development` идемпотентно создаёт demo-тенант (`demo`), demo-пользователей и включает core модули для локальной среды. Пароль задаётся через `RUSTOK_DEV_SEED_PASSWORD` (дефолт: `dev-password-123`).

### Email (password reset)

Кастомный `EmailService` (`src/services/email.rs`) на базе `lettre`. Конфигурируется через `settings.rustok.email`. При `enabled: false` — безопасный no-send fallback с логированием. Целевое решение — миграция на Loco Mailer API (см. `LOCO_FEATURE_SUPPORT.md` п. 3.1).

### Build pipeline

`BuildService::request_build` публикует `BuildRequested` через `BuildEventPublisher`. `EventBusBuildEventPublisher` пересылает в `DomainEvent::BuildRequested`, нoop-паблишер логирует skipped dispatch.
