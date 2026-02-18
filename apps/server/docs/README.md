# server docs

В этой папке хранится документация модуля `apps/server`.

## Документы

- [`library-stack.md`](./library-stack.md) — основные backend-библиотеки сервера и их роль (framework, HTTP, ORM, GraphQL, runtime, observability).
- [`event-transport.md`](./event-transport.md) — как работает конфигурация и runtime-пайплайн транспорта событий.
- [`loco/README.md`](./loco/README.md) — Loco-specific контекст, workflow для агентов и freshness-политика upstream snapshot.
- [`upstream-libraries/README.md`](./upstream-libraries/README.md) — локальный snapshot актуальной внешней документации по ключевым crate сервера.
- Cleanup/maintenance: background cleanup task (`cargo loco task --name cleanup --args "sessions"`) removes expired sessions; app `truncate` hook now performs ordered deletion of server foundation tables (`release`, `build`, `tenant_modules`, `sessions`, `users`, `tenants`).
- Auth/password reset: GraphQL `forgot_password` now dispatches reset emails via SMTP (`rustok.email` settings, credentials optional for local relay) with safe no-send fallback when email delivery is disabled.
- Dev onboarding: `seed_development` creates/updates an idempotent demo tenant (`demo`), demo users, and enables core modules for local environments.
