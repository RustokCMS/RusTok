# Обзор архитектуры

RusToK сейчас представляет собой событийный modular monolith вокруг `apps/server`, platform modules, shared libraries и capability/support crates.

Этот документ фиксирует **текущее состояние кода**, а не исторические или целевые варианты.

## Короткая модель

- `apps/server` — composition root и transport/runtime host.
- platform modules делятся только на `Core` и `Optional`.
- write-side живёт в нормализованных таблицах и сервисах доменных модулей.
- downstream-эффекты идут через event runtime и transactional outbox.
- read-side и быстрые выборки обслуживаются через `rustok-index`.

## Архитектурные роли

### 1. Приложения

- `apps/server` — HTTP, GraphQL, auth/session, RBAC wiring, health, metrics, build/runtime orchestration
- `apps/admin` — основной Leptos admin host
- `apps/storefront` — основной Leptos storefront host
- `apps/next-admin`, `apps/next-frontend` — headless/experimental Next.js hosts

### 2. Platform modules

Только два статуса:

- `Core`
- `Optional`

#### Core modules

- `auth`
- `cache`
- `email`
- `index`
- `outbox`
- `tenant`
- `rbac`

#### Optional modules

- `content`
- `commerce`
- `blog`
- `forum`
- `pages`
- `media`
- `workflow`

### 3. Shared libraries и support/capability crates

- shared libraries: `rustok-core`, `rustok-api`, `rustok-events`, `rustok-storage`, `rustok-test-utils`
- support/capability crates: `rustok-telemetry`, `rustok-iggy`, `rustok-iggy-connector`, `rustok-mcp`, `alloy`, `alloy-scripting`, `flex`

Важно: `crate` и `module` не являются синонимами. Не каждый crate автоматически становится platform module.

## Runtime composition

Текущий composition-root строится так:

1. `modules.toml` задаёт состав platform modules.
2. `apps/server/src/modules/mod.rs` собирает `ModuleRegistry`.
3. `apps/server/build.rs` генерирует wiring для optional modules.
4. `apps/server/src/modules/manifest.rs` сверяет manifest и runtime contract.
5. `apps/server/src/services/app_runtime.rs` и соседние runtime services поднимают event/runtime bootstrap.

Это означает:

- `ModuleRegistry` — composition point, а не отдельная taxonomy;
- bootstrap/runtime wiring не создаёт “третий тип модулей”;
- `rustok-outbox` остаётся `Core` module, даже если сервер дополнительно использует его в event runtime bootstrap.

## Event flow

Канонический write/read path сейчас такой:

1. transport layer получает запрос;
2. tenant/auth/RBAC контекст собирается в `apps/server`;
3. доменный сервис выполняет write-side изменения;
4. события публикуются через transactional outbox;
5. consumers и indexers обновляют downstream state;
6. storefront/admin/read API читают уже подготовленные read models или доменные данные.

## API модель

RusToK использует гибридную API-схему:

- GraphQL — основной UI-facing и platform-facing контракт;
- REST — integrations, health, operational endpoints и отдельные resource flows;
- WebSocket/subscription paths — для live runtime сценариев;
- OpenAPI — для REST surface и operational discovery.

## Alloy и capability boundaries

`alloy` и `alloy-scripting` не входят в taxonomy `Core/Optional` platform modules.

Они остаются capability-слоем:

- могут использоваться server runtime, workflow и MCP;
- не управляются как tenant-toggle modules;
- не должны описываться как обычные доменные модули.

## Definition of done для архитектурных изменений

1. Состав platform modules согласован между `modules.toml`, registry и verification docs.
2. Изменение boundaries отражено в `docs/modules/*` и `docs/index.md`.
3. Если меняется runtime contract, обновлены `docs/architecture/*` и локальные README/docs затронутых компонентов.
4. Для нетривиальных boundary changes зафиксирован ADR.
