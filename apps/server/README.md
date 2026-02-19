# apps/server — RusToK Server

## Что это такое

`apps/server` — это **голая Loco.rs-платформа**: минимальный бинарник, который целиком строится на Loco.rs как каркасе и не содержит собственной бизнес-логики.

Сам сервер — это **хост-оболочка**: он настраивает и запускает всё loco-инфраструктурное (boot, конфиги, auth, migrations, workers, mailer, storage, tasks, initializers), подключает клиентские приложения (admin UI, storefront) к единому API и **поверх этого каркаса регистрирует доменные модули** — `rustok-content`, `rustok-commerce`, `rustok-blog`, `rustok-forum`, `rustok-pages`.

```
┌─────────────────────────────────────────────────────────────┐
│                       apps/server                           │
│                                                             │
│   ┌─────────────────────────────────────────────────────┐   │
│   │              Loco.rs platform shell                 │   │
│   │  boot · config · auth · migrations · tasks ·       │   │
│   │  initializers · workers · mailer · storage ·       │   │
│   │  middleware (tenant, rate-limit) · seeding          │   │
│   └─────────────────────────────────────────────────────┘   │
│                            │                                │
│              регистрирует поверх каркаса                    │
│                            │                                │
│   ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────┐  │
│   │ commerce │ │ content  │ │   blog   │ │ forum/pages  │  │
│   │  module  │ │  module  │ │  module  │ │   modules    │  │
│   └──────────┘ └──────────┘ └──────────┘ └──────────────┘  │
│                                                             │
│   ┌─────────────────────────────────────────────────────┐   │
│   │          GraphQL + REST API endpoints               │   │
│   │   admin/storefront UI clients   ←→   integrations  │   │
│   └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

## Роль в системе

| Что | Где |
|-----|-----|
| Loco.rs lifecycle & boot | `src/app.rs` — `impl Hooks for App` |
| Конфигурация | `config/development.yaml`, `config/test.yaml`, секция `settings.rustok.*` |
| Миграции БД | `migration/` — SeaORM-миграции всех модулей |
| REST-контроллеры | `src/controllers/` — health, metrics, auth, swagger, domain REST |
| GraphQL endpoint | `src/controllers/graphql.rs` + `src/graphql/` — единая схема для admin/storefront |
| Middleware | `src/middleware/` — tenant resolution + rate-limit |
| Доменные модули | `src/modules/mod.rs` — `build_registry()` регистрирует все модули |
| Сервисы | `src/services/` — event transport, auth, email, build pipeline |
| Задачи | `src/tasks/` — `cleanup` task |
| Инициализаторы | `src/initializers/` — telemetry |
| Seeding | `src/seeds/` — demo tenant + users для dev/test |

## Что подключается поверх

### Доменные модули (`src/modules/mod.rs`)

```rust
pub fn build_registry() -> ModuleRegistry {
    ModuleRegistry::new()
        .register(ContentModule)
        .register(CommerceModule)
        .register(BlogModule)
        .register(ForumModule)
        .register(PagesModule)
}
```

Каждый модуль (`crates/rustok-*`) реализует `RusToKModule` и подключает свои миграции, GraphQL-резолверы и event listeners. Сервер не знает о внутренней логике модулей — только регистрирует их.

### Клиентские приложения

Сервер обслуживает следующие frontend-клиенты через единый API:

| Клиент | Протокол | Назначение |
|--------|----------|-----------|
| `apps/admin` (Leptos CSR) | GraphQL | Панель управления — CRUD сущностей, tenant management, RBAC |
| `apps/next-admin` (Next.js) | GraphQL + REST | Альтернативная admin-панель |
| `apps/storefront` (Leptos SSR) | GraphQL | Покупательская витрина |
| `apps/next-frontend` (Next.js) | GraphQL | Альтернативная витрина |
| Интеграции | REST | Внешние сервисы, webhooks, batch операции |

## Loco.rs — что используем

`apps/server` использует Loco.rs как есть, не заменяя его компоненты:

- **Config** — `config/*.yaml` + `settings.rustok` для проектных настроек
- **Auth** — JWT access/refresh, Users model, password hashing
- **Migrations** — SeaORM + Loco Migrator
- **Tasks** — `cargo loco task`
- **Initializers** — lifecycle hook перед монтированием роутов
- **Workers** — `connect_workers` hook (outbox relay worker)
- **Mailer** — цель: мигрировать на Loco Mailer API (сейчас кастомный `lettre`)
- **Storage** — цель: единый Loco storage layer

Подробнее о матрице Loco vs самопис: [`docs/LOCO_FEATURE_SUPPORT.md`](./docs/LOCO_FEATURE_SUPPORT.md)

## Транспорт событий

Сервер поддерживает три режима транспорта через `settings.rustok.events.transport` или `RUSTOK_EVENT_TRANSPORT`:

| Транспорт | Описание |
|-----------|----------|
| `memory` | In-memory MPSC bus (локальная разработка) |
| `outbox` | Outbox pattern — запись в БД + фоновый relay worker |
| `iggy` | Streaming через Iggy (L2) |

При неверном значении сервер завершит старт с ошибкой валидации.

## Паспорт компонента

- **Роль:** Главный backend RusToK — API-хост, Loco-каркас, оркестрация модулей
- **Ответственность:** HTTP-сервер, auth, tenant middleware, event runtime, migrations, seeding, tasks
- **Взаимодействует с:**
  - `crates/rustok-core` — инфраструктурное ядро (события, кэш, registry)
  - `crates/rustok-{content,commerce,blog,forum,pages}` — доменные модули
  - `crates/rustok-{outbox,iggy,telemetry}` — инфра-модули
  - `apps/admin`, `apps/next-admin`, `apps/storefront`, `apps/next-frontend` — UI-клиенты
- **Точки входа:**
  - `src/main.rs` — точка запуска
  - `src/app.rs` — Loco hooks lifecycle
  - `src/modules/mod.rs` — регистрация модулей
  - `src/controllers/` — API endpoints
- **Локальная документация:** [`./docs/`](./docs/)
- **Глобальная документация:** [`/docs/`](../../docs/)
