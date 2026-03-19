# Module & Application Registry

This document provides a comprehensive map of all components within the RusToK ecosystem, including their relationships and responsibilities.

## High-Level Architecture

```mermaid
graph TD
    subgraph Applications
        SERVER["apps/server — Loco Server"]
        ADMIN["apps/admin — Leptos Admin (primary)"]
        SF["apps/storefront — Leptos Storefront (primary)"]
        NEXT_ADMIN["apps/next-admin — Next.js Admin (experimental headless)"]
        NEXT_SF["apps/next-frontend — Next.js Storefront (experimental headless)"]
        ADMIN_PKGS["apps/next-admin/packages/* — @rustok/*-admin"]
        SF_PKGS["apps/next-frontend/packages/* — @rustok/*-frontend"]
    end

    subgraph Domain Modules
        COMMERCE[crates/rustok-commerce]
        CONTENT[crates/rustok-content]
        BLOG[crates/rustok-blog]
        FORUM[crates/rustok-forum]
        PAGES[crates/rustok-pages]
        INDEX[crates/rustok-index]
        RBAC[crates/rustok-rbac]
        TENANT[crates/rustok-tenant]
        ALLOY[crates/alloy-scripting]
        WORKFLOW[crates/rustok-workflow]
        MEDIA[crates/rustok-media]
    end

    subgraph Platform Core Crates
        CORE[crates/rustok-core]
        EVENTS[crates/rustok-events]
        STORAGE[crates/rustok-storage]
        OUTBOX["crates/rustok-outbox — Core Infra"]
        CACHE[crates/rustok-cache]
        IGGY[crates/rustok-iggy]
        IGGY_CONN[crates/rustok-iggy-connector]
        MCP[crates/rustok-mcp]
        TELEMETRY[crates/rustok-telemetry]
        TEST_UTILS[crates/rustok-test-utils]
        FLEX[crates/flex]
    end

    subgraph Frontend Libraries - internal custom
        L_AUTH[crates/leptos-auth]
        L_UI[crates/leptos-ui]
        L_GRAPHQL[crates/leptos-graphql]
        L_FORMS[crates/leptos-forms]
        L_HOOKFORM[crates/leptos-hook-form]
        L_TABLE[crates/leptos-table]
        L_ZOD[crates/leptos-zod]
        L_ZUSTAND[crates/leptos-zustand]
        L_PAGINATION[crates/leptos-shadcn-pagination]
    end

    SERVER --> COMMERCE
    SERVER --> CONTENT
    SERVER --> RBAC
    SERVER --> TENANT
    SERVER --> INDEX
    SERVER --> BLOG
    SERVER --> FORUM
    SERVER --> PAGES
    SERVER --> ALLOY
    SERVER --> WORKFLOW

    ADMIN --> L_AUTH
    ADMIN --> L_UI
    ADMIN --> L_GRAPHQL
    ADMIN --> L_HOOKFORM
    ADMIN --> L_TABLE
    ADMIN --> L_PAGINATION
    SF --> L_UI
    SF --> L_GRAPHQL

    COMMERCE --> CORE
    COMMERCE --> EVENTS
    COMMERCE --> OUTBOX
    COMMERCE --> MEDIA
    CONTENT --> CORE
    CONTENT --> OUTBOX
    CONTENT --> MEDIA
    BLOG --> CONTENT
    FORUM --> CONTENT
    PAGES --> CORE
    INDEX --> CORE
    OUTBOX --> IGGY
    IGGY --> IGGY_CONN
    ALLOY --> CORE
    WORKFLOW --> CORE
    WORKFLOW --> ALLOY
    CACHE --> CORE
    STORAGE --> CORE
    MEDIA --> CORE
    MEDIA --> STORAGE
    MEDIA --> OUTBOX
    FLEX --> CORE

    %% Leptos UI lives inside module crate folders as sub-crates:
    BLOG --> BLOG_ADMIN["admin/ (rustok-blog-admin)"]
    BLOG --> BLOG_SF["storefront/ (rustok-blog-storefront)"]
    BLOG_ADMIN -. "active via feature flags" .-> ADMIN
    BLOG_SF -. "active via feature flags" .-> SF

    %% Next.js UI lives in apps/*/packages/ as modular npm packages:
    ADMIN_PKGS -. imports .-> BLOG
    SF_PKGS -. imports .-> BLOG
    NEXT_ADMIN --> ADMIN_PKGS
    NEXT_SF --> SF_PKGS

    Domain Modules -.-> TELEMETRY
```

## Component Directory

### Applications (`apps/`)

| Path | Name | Description |
|------|------|-------------|
| `apps/server` | **Server** | Main API server built on Loco.rs. Orchestrates all domain modules. ([CRATE_API](../../apps/server/CRATE_API.md)) |
| `apps/admin` | **Leptos Admin** (primary) | Back-office management interface built with Leptos (CSR/WASM). Participates in auto-deploy on module install/uninstall. ([CRATE_API](../../apps/admin/CRATE_API.md)) |
| `apps/storefront` | **Leptos Storefront** (primary) | Customer-facing web interface built with Leptos (SSR). Participates in auto-deploy on module install/uninstall. ([CRATE_API](../../apps/storefront/CRATE_API.md)) |
| `apps/next-admin` | **Next.js Admin** | Alternative React-based admin interface. UI packages live in `packages/`. Manual rebuild; not in module pipeline. ([CRATE_API](../../apps/next-admin/CRATE_API.md)) |
| `apps/next-frontend` | **Next.js Storefront** | Alternative React-based storefront. UI packages live in `packages/`. Manual rebuild; not in module pipeline. ([CRATE_API](../../apps/next-frontend/CRATE_API.md)) |

### Core Platform Crates (`crates/`)

#### Модули-библиотеки уровня 0 (leaf — без зависимости от core)

Предоставляют чистые типы и трейты. Core зависит от них и ре-экспортирует.

| Path | Name | Description |
|------|------|-------------|
| `crates/rustok-events` | **Events Contracts** | Модуль-библиотека (leaf). Stable import point for `DomainEvent`/`EventEnvelope`. Core re-exports. ([CRATE_API](../../crates/rustok-events/CRATE_API.md)) |
| `crates/rustok-telemetry` | **Telemetry** | Модуль-библиотека (leaf). Observability setup (OTLP, Tracing, Prometheus metrics). Core re-exports. ([CRATE_API](../../crates/rustok-telemetry/CRATE_API.md)) |
| `crates/rustok-storage` | **Storage** | Модуль-библиотека (leaf). `StorageBackend` async trait + `LocalStorage` backend, `StorageService` wrapper, `StorageConfig`. ([docs](../../crates/rustok-storage/docs/README.md)) |

#### Модуль-агрегатор (уровень 1)

| Path | Name | Description |
|------|------|-------------|
| `crates/rustok-core` | **Core (critical)** | Модуль-библиотека (агрегатор). Re-exports leaf contracts, provides `CacheBackend`, `ModuleRegistry`, `RusToKModule`, RBAC primitives, i18n, `SecurityContext`, circuit breaker. Содержит **Flex** — набор типов, валидаторов и migration-хелперов для кастомных полей (`field_schema.rs`, `HasCustomFields`). ([CRATE_API](../../crates/rustok-core/CRATE_API.md)) |

> **Flex** — часть модуля `rustok-core`. Сейчас — набор типов и хелперов (модуль-библиотека). Данные живут внутри модуля-потребителя.
> Режим **Attached** (кастомные поля для сущностей) — в core. Режим **Standalone** (`flex`) — запланирован.
> План реализации: [`docs/architecture/flex.md`](../architecture/flex.md).

#### Инфраструктурные модули-библиотеки

| Path | Name | Description |
|------|------|-------------|
| `crates/rustok-cache` | **Cache** | Redis connection lifecycle, `CacheModule` + `CacheService`. `CacheBackend` (Moka + Redis + Fallback): circuit breaker, anti-stampede coalescing, negative cache, Redis pub/sub invalidation, metrics. ([docs](../../crates/rustok-cache/docs/README.md)) |
| `crates/rustok-outbox` | **Outbox (Core, critical)** | Модуль-библиотека. Core event delivery (`TransactionalEventBus`). Initialized via `build_event_runtime()`. ([CRATE_API](../../crates/rustok-outbox/CRATE_API.md)) |
| `crates/rustok-iggy` | **Iggy Transport** | L2 streaming `EventTransport` implementation with serialization, topology, DLQ, replay. ([CRATE_API](../../crates/rustok-iggy/CRATE_API.md)) |
| `crates/rustok-iggy-connector` | **Iggy Connector** | Embedded/Remote mode switching, connection lifecycle, message I/O. ([CRATE_API](../../crates/rustok-iggy-connector/CRATE_API.md)) |
| `crates/rustok-mcp` | **MCP** | MCP adapter crate with embedded `rustok-mcp-server` binary. Exposes RusToK tools/resources via the MCP protocol using the `rmcp` SDK. ([CRATE_API](../../crates/rustok-mcp/CRATE_API.md)) |
| `crates/rustok-test-utils` | **Test Utils** | Shared testing helpers and mocks. `[dev-dependencies]` only — never in production binary. ([CRATE_API](../../crates/rustok-test-utils/CRATE_API.md)) |

### Полноценные модули (`crates/`)

Реализуют `RusToKModule`, регистрируются через `ModuleRegistry` в `apps/server`.
Имеют таблицы, entities, бизнес-логику.

#### Core-модули (уровень 2 — `ModuleKind::Core`, нельзя отключить)

| Path | Name | Kind | Depends on |
|------|------|------|-----------|
| `crates/rustok-tenant` | **Tenant** | `Core` (mandatory, critical) | `rustok-core` ([CRATE_API](../../crates/rustok-tenant/CRATE_API.md)) |
| `crates/rustok-rbac` | **RBAC** | `Core` (mandatory, critical) | `rustok-core`. Владеет resolver contracts, permission policy и Casbin-backed authorization runtime; `apps/server` держит только adapter/wiring слой и `RbacService`. ([CRATE_API](../../crates/rustok-rbac/CRATE_API.md)) |
| `crates/rustok-index` | **Index** | `Core` (mandatory, critical) | `rustok-core` ([CRATE_API](../../crates/rustok-index/CRATE_API.md)) |
| `crates/rustok-media` | **Media** | `Core` (mandatory, feature `mod-media`) | `rustok-core`, `rustok-storage`. `MediaService`: upload/get/list/delete + translations. SeaORM entities `media` + `media_translations`. ([docs](../../crates/rustok-media/docs/README.md)) |

#### Optional-модули (уровень 3 — `ModuleKind::Optional`, toggle per-tenant)

| Path | Name | Kind | Depends on |
|------|------|------|-----------|
| `crates/rustok-content` | **Content** | `Optional` | `rustok-core`, `rustok-media` ([CRATE_API](../../crates/rustok-content/CRATE_API.md)) |
| `crates/rustok-commerce` | **Commerce** | `Optional` | `rustok-core`, `rustok-media` ([CRATE_API](../../crates/rustok-commerce/CRATE_API.md)) |
| `crates/rustok-blog` | **Blog** | `Optional` | `rustok-content` ([CRATE_API](../../crates/rustok-blog/CRATE_API.md)) |
| `crates/rustok-forum` | **Forum** | `Optional` | `rustok-content` ([CRATE_API](../../crates/rustok-forum/CRATE_API.md)) |
| `crates/rustok-pages` | **Pages** | `Optional` | `rustok-core` ([CRATE_API](../../crates/rustok-pages/CRATE_API.md)) |
| `crates/alloy-scripting` | **Alloy Scripting** | `Optional` | `rustok-core` (registered via `AlloyModule` in `apps/server/src/modules/alloy.rs`) |
| `crates/rustok-workflow` | **Workflow** | `Optional` | `rustok-core`, `alloy-scripting`. Визуальная автоматизация на платформенной очереди. Горизонтальный модуль. ([docs](../../crates/rustok-workflow/docs/README.md) · [CRATE_API](../../crates/rustok-workflow/CRATE_API.md) · [arch](../architecture/workflow.md)) |

> **4-уровневая архитектура платформы:**
> - Уровень 0 (модули-библиотеки, leaf): `rustok-events`, `rustok-telemetry`, `rustok-storage`
> - Уровень 1 (модуль-агрегатор): `rustok-core` (зависит от leaf, ре-экспортирует их)
> - Уровень 2 (полноценные Core-модули, всегда активны): `rustok-tenant`, `rustok-rbac`, `rustok-index`, `rustok-media`
> - Уровень 3 (полноценные Optional-модули, toggle per-tenant): `content`, `commerce`, `blog`, `forum`, `pages`, `alloy-scripting`, `workflow`
>
> Обязательный базис платформы: `rustok-core`, `rustok-outbox`, `rustok-telemetry`, `rustok-tenant`, `rustok-rbac`, `rustok-index` + инфраструктурные модули (`rustok-cache`, `rustok-events`).
>
> **Граница подвижна:** модуль-библиотека может получить таблицы и стать полноценным модулем.

### Module UI Architecture (updated 2026-03-18)

> [!IMPORTANT]
> See [ADR: Dual UI Strategy](../../DECISIONS/2026-03-17-dual-ui-strategy-next-batteries-included.md) for rationale.

#### Leptos UI — separate sub-crates, publishable to crates.io

Each module crate contains Leptos UI as dedicated sub-crates:

```text
crates/rustok-<module>/
  Cargo.toml           # rustok-<module> (backend, publishable)
  src/
    lib.rs             # domain types (always compiled)
    services/          # backend
  admin/
    Cargo.toml         # rustok-<module>-admin → crates.io
    src/lib.rs         # Leptos admin components + registration fn
  storefront/
    Cargo.toml         # rustok-<module>-storefront → crates.io
    src/lib.rs         # Leptos SSR components + registration fn
```

`apps/admin/Cargo.toml` lists each `rustok-<m>-admin` as a dependency.
Auto-installation via marketplace triggers Leptos WASM rebuild automatically (BuildExecutor).

#### Next.js UI — modular packages inside apps

```text
apps/next-admin/
  packages/
    blog/              # @rustok/blog-admin
    commerce/          # @rustok/commerce-admin
  src/                 # app itself, imports from packages/*
  package.json

apps/next-frontend/
  packages/
    blog/              # @rustok/blog-frontend
    commerce/          # @rustok/commerce-frontend
  src/
  package.json
```

To remove a module from Next.js: delete `packages/<module>/`, remove from `package.json`, then `npm install && npm run build`. Not part of the auto-install pipeline.

#### Legacy paths

| Old path | Migrate to |
|----------|------------|
| `crates/rustok-blog/ui/admin` | `crates/rustok-blog/admin/` (Leptos sub-crate) |
| `crates/rustok-blog/ui/frontend` | `crates/rustok-blog/storefront/` (Leptos sub-crate) |
| `apps/next-admin/src/features/blog/` | `apps/next-admin/packages/blog/` |
| `apps/next-frontend/src/features/blog/` | `apps/next-frontend/packages/blog/` |

### Internal Frontend Libraries (`crates/`)

All `leptos-*` and `tailwind-*` crates are **internal custom libraries** written and maintained by the RusToK team.
They are not published to crates.io. Treat them as first-party code — changes here affect all consuming apps.

| Path | Name | Used by | Description |
|------|------|---------|-------------|
| `crates/leptos-ui` | **Leptos UI** | `apps/admin`, `apps/storefront` | Shared UI component library. |
| `crates/leptos-auth` | **Leptos Auth** | `apps/admin` | Authentication hooks and components. |
| `crates/leptos-forms` | **Leptos Forms** | `apps/admin` | Low-level form abstractions. |
| `crates/leptos-graphql` | **Leptos GraphQL** | `apps/admin`, `apps/storefront` | Thin GraphQL client wrapper (request shape, headers, error mapping). |
| `crates/leptos-hook-form` | **Leptos Hook Form** | `apps/admin` | Hook-form style bindings (React-hook-form inspired). |
| `crates/leptos-shadcn-pagination` | **Leptos Pagination** | `apps/admin` | Pagination components compatible with shadcn design. |
| `crates/leptos-table` | **Leptos Table** | `apps/admin` | Data table primitives. |
| `crates/leptos-zod` | **Leptos Zod** | `apps/admin` | Zod-like validation helpers. |
| `crates/leptos-zustand` | **Leptos Zustand** | `apps/admin` | Lightweight state management utilities. |
| `crates/utoipa-swagger-ui-vendored` | **Swagger UI** | `apps/server` | Vendored Swagger UI static assets. |
| `crates/rustok-cache` | **Cache** | `apps/server` | Redis/Moka cache backends, `CacheModule`, `CacheService`. Выделен из `rustok-core`. |
| `crates/rustok-storage` | **Storage** | `rustok-media`, `apps/server` | Leaf crate: `StorageBackend` async trait + `LocalStorage` impl, `StorageService` wrapper. ([docs](../../crates/rustok-storage/docs/README.md)) |
| `crates/rustok-media` | **Media** | `apps/server`, `rustok-content`, `rustok-commerce` | Core module: `MediaService`, SeaORM entities `media` + `media_translations`. Feature: `mod-media`. ([docs](../../crates/rustok-media/docs/README.md)) |
| `crates/flex` | **Flex Contracts** *(Phase 4.5, in progress)* | `apps/server` (Attached mode) | Optional crate для выноса Flex attached-mode контрактов (`FieldDefinitionService`, `FieldDefRegistry`, DTOs). Standalone режим планируется следующим этапом. |

## Maintenance Rule

> [!IMPORTANT]
> This registry must be kept up to date. AI Agents are required to update the **Mermaid diagram** and **Component Directory** whenever a new crate or application is added, renamed, or significantly restructured.
> Also update [`docs/index.md`](../index.md) when this registry changes.
>
> Перед изменением любого `crates/rustok-*` необходимо проверить и обновить соответствующий `CRATE_API.md`, если изменился публичный контракт (модули, сигнатуры, события, зависимости).
