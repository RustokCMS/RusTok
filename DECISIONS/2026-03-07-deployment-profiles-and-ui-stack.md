# Deployment Profiles и выбор UI-стека

- Date: 2026-03-07
- Status: Proposed

## Context

RusTok поддерживает два UI-стека параллельно:

- **Leptos** (Rust) — admin (`apps/admin`) + storefront (`apps/storefront`)
- **Next.js** (TypeScript) — admin (`apps/next-admin`) + storefront (`apps/next-frontend`)

Обе админки и оба storefront'а делают **одно и то же** — используют единый
GraphQL API на бэкенде. Разница — в deployment-модели:

| | Leptos | Next.js |
|---|---|---|
| **Язык** | Rust / WASM | TypeScript / Node.js |
| **Монолит** | Может компилироваться в один бинарник с сервером | Отдельный Node.js-процесс всегда |
| **SSR** | Axum (тот же сервер) | Node.js (отдельный сервер) |
| **Admin** | CSR / WASM, раздаётся через nginx или axum static | Standalone Next.js server |
| **Аналог WordPress** | Да — один бинарник, установил и работает | Нет — минимум 2 процесса |

Проблема: **нельзя запускать две админки одновременно**. Оператор должен выбрать
один стек при установке/сборке платформы.

## Decision

### 1. Расширить `deployment_profile` в `modules.toml`

Вместо двух значений (`monolith | headless`) вводим **три** deployment preset'а,
каждый из которых определяет и UI-стек, и способ деплоя:

```toml
[build]
deployment_profile = "monolith"  # monolith | headless-leptos | headless-next
```

| Profile | Server | Admin | Storefront | Кол-во процессов | Аналог |
|---|---|---|---|---|---|
| `monolith` | Axum (Rust) | Leptos (WASM, встроен) | Leptos (SSR, встроен) | **1** | WordPress |
| `headless-leptos` | Axum (API only) | Leptos (WASM, отдельно) | Leptos (SSR, отдельно) | 3 | — |
| `headless-next` | Axum (API only) | Next.js (Node.js) | Next.js (Node.js) | 3 | Strapi + Next.js |

### 2. Как это выглядит для оператора

#### При первоначальной установке

```bash
# Вариант 1: Монолит (как WordPress — один бинарник)
rustok init --profile monolith
# → Собирает один бинарник: server + admin + storefront
# → Запускается одной командой: ./rustok-server
# → Admin доступен на /admin, storefront на /

# Вариант 2: Headless с Leptos
rustok init --profile headless-leptos
# → Собирает 3 бинарника: server, admin (WASM), storefront (SSR)
# → Деплоятся отдельно, могут быть в разных регионах

# Вариант 3: Headless с Next.js
rustok init --profile headless-next
# → Собирает server (Rust), admin и storefront через npm
# → Server = Axum API, admin/storefront = Node.js
```

#### При смене профиля (миграция)

```bash
# Переключить с headless-next на monolith
rustok switch-profile monolith
# → Пересборка: сервер с встроенными Leptos admin + storefront
# → Данные (БД, tenant_modules, users) — без изменений
# → Следующий деплой запускает один бинарник вместо трёх сервисов
```

### 3. Что определяет профиль

Профиль определяет **только способ сборки и деплоя**. Всё остальное
(модули, tenant_modules, GraphQL API, RBAC) — идентично:

```
                    ┌─────────────────────────────────────────┐
                    │          Общее для всех профилей         │
                    │                                          │
                    │  • GraphQL API (async-graphql)           │
                    │  • ModuleRegistry + ModuleLifecycle      │
                    │  • RBAC, Tenants, Events, Outbox         │
                    │  • modules.toml (состав модулей)         │
                    │  • tenant_modules (toggle per tenant)    │
                    │  • Маркетплейс (один и тот же каталог)   │
                    └────────────────┬────────────────────────┘
                                     │
                 ┌───────────────────┼───────────────────┐
                 │                   │                   │
                 ▼                   ▼                   ▼
          ┌──────────┐       ┌──────────────┐    ┌──────────────┐
          │ monolith │       │headless-     │    │headless-     │
          │          │       │leptos        │    │next          │
          ├──────────┤       ├──────────────┤    ├──────────────┤
          │ 1 binary │       │ 3 binaries   │    │ 1 binary     │
          │ Axum +   │       │ Axum API     │    │ + 2 Node.js  │
          │ Leptos   │       │ Leptos WASM  │    │ Axum API     │
          │ Admin +  │       │ Leptos SSR   │    │ Next Admin   │
          │ Store    │       │              │    │ Next Store   │
          └──────────┘       └──────────────┘    └──────────────┘
```

### 4. Реализация через Cargo features

```toml
# apps/server/Cargo.toml
[features]
default = ["monolith"]

# Встраивает Leptos admin (CSR/WASM assets) и storefront (SSR) в сервер
monolith = ["leptos-admin-embed", "leptos-storefront-embed"]

# API-only — без UI, для headless-* профилей
headless = []

# Отдельные фичи для встраивания
leptos-admin-embed = ["dep:admin-assets"]
leptos-storefront-embed = ["dep:leptos-storefront"]
```

Build pipeline читает `deployment_profile` из `modules.toml` и выбирает features:

```bash
# monolith
cargo build -p rustok-server --release --features monolith

# headless-leptos или headless-next
cargo build -p rustok-server --release --features headless
# + отдельно собирает admin и storefront
```

### 5. Маркетплейс — profile-agnostic

Маркетплейс модулей **не зависит от профиля**. Модуль публикуется один раз
и работает в любом профиле, потому что:

- Backend-часть (trait `RusToKModule`, GraphQL, миграции) — **одинаковая**.
- UI-часть (admin компоненты, storefront виджеты) — **два варианта в одном crate**:

```
rustok-blog/
├── src/                    # Backend (работает везде)
├── ui/
│   ├── leptos/             # Для monolith и headless-leptos
│   │   ├── admin/
│   │   └── storefront/
│   └── next/               # Для headless-next
│       ├── admin/           # @rustok/blog-admin npm package
│       └── frontend/        # @rustok/blog-frontend npm package
```

При сборке build pipeline включает только нужную UI-часть.

### 6. Рекомендации по выбору профиля

| Сценарий | Рекомендация |
|---|---|
| Self-hosted, один сервер, хочется как WordPress | `monolith` |
| Self-hosted, нужен горизонтальный scaling | `headless-leptos` |
| Команда на TypeScript/React | `headless-next` |
| Multi-region, CDN для storefront | `headless-leptos` или `headless-next` |
| Максимальная производительность | `monolith` (один процесс, нет HTTP между сервисами) |
| Разработка, быстрый старт | `monolith` (одна команда) |

## Consequences

### Позитивные

- **Чёткий выбор** при установке — нет путаницы "какую админку использовать".
- **Монолит как WordPress** — один бинарник, минимум инфраструктуры.
- **Гибкость** — можно переключить профиль при необходимости.
- **Маркетплейс не зависит от профиля** — модуль работает везде.
- **Планы и документация** автоматически покрывают все профили, потому что
  разница только в build/deploy, а не в бизнес-логике.

### Негативные

- **Два UI-стека поддерживать дороже** — каждый модуль должен иметь
  UI-компоненты для обоих стеков (или только для одного, если второй не нужен).
- **Тестирование** — CI должен проверять все три профиля.
- **Монолит-режим для Leptos storefront SSR** требует интеграции с Axum,
  что усложняет routing (admin routes vs storefront routes на одном порту).

### Follow-up

1. Добавить `ui_stack` поле в `modules.toml` (или вывести из `deployment_profile`).
2. Добавить Cargo features `monolith` / `headless` в `apps/server/Cargo.toml`.
3. Обновить `DeploymentProfile` enum: `Monolith` / `HeadlessLeptos` / `HeadlessNext`.
4. Обновить build pipeline: генерация `cargo build` команды на основе профиля.
5. Обновить Makefile: `make build-monolith`, `make build-headless-leptos`, `make build-headless-next`.
6. Документировать в README: таблица профилей с инструкциями по установке.
