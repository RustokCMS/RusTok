# UI Documentation Hub

This section documents frontend applications and shared UI integration patterns used in RusToK.

## Current frontend landscape

RusToK has four UI applications across two stacks:

### Primary (Leptos) — авто-деплой при install/uninstall модулей

- `apps/admin` — **primary Leptos admin panel** (CSR/WASM). Участвует в пересборке WASM при установке/удалении модулей.
- `apps/storefront` — **primary Leptos storefront** (SSR). Участвует в пересборке WASM при установке/удалении модулей.

Leptos UI-код модулей находится в отдельных publishable sub-crates рядом с бекенд-крейтом (`admin/`, `storefront/`).

### Experimental headless (Next.js) — ручная сборка

- `apps/next-admin` — альтернативная Next.js админка (headless-режим). Пересборка **вручную** (`npm run build`). Не участвует в module install pipeline.
- `apps/next-frontend` — альтернативный Next.js storefront (headless-режим). Пересборка **вручную**. Не участвует в module install pipeline.

Next.js UI-код модулей находится в виде отдельных npm-пакетов внутри директории самого приложения (`apps/next-admin/packages/<module>/` и `apps/next-frontend/packages/<module>/`).

> See [ADR: Dual UI Strategy — Next.js modular packages](../../DECISIONS/2026-03-17-dual-ui-strategy-next-batteries-included.md) for the rationale.

For platform-wide app ownership and dependencies, see [`docs/modules/registry.md`](../modules/registry.md).

## Documents in this section

- [GraphQL Architecture](./graphql-architecture.md) — client-side GraphQL conventions.
- [Admin ↔ Server Connection Quickstart](./admin-server-connection-quickstart.md) — backend connection and env setup.
- [Storefront](./storefront.md) — storefront-specific UI notes.
- [Rust UI Component Catalog](./rust-ui-component-catalog.md) — reusable components and crates.

## App-level documentation

- [Next.js Admin README](../../apps/next-admin/README.md)
- [Next.js Admin RBAC Navigation](../../apps/next-admin/docs/nav-rbac.md)
- [Next.js Admin Clerk setup](../../apps/next-admin/docs/clerk_setup.md)
- [Next.js Admin Theming](../../apps/next-admin/docs/themes.md)
- [Leptos Admin docs](../../apps/admin/docs/README.md)
- [Leptos Storefront README](../../apps/storefront/README.md)
- [Next.js Storefront docs](../../apps/next-frontend/docs/README.md)

## Maintenance notes

When frontend architecture, routing, UI contracts, or API integration changes:

1. Update the relevant app-level docs in `apps/*`.
2. Update the corresponding document in `docs/UI/`.
3. Ensure [`docs/index.md`](../index.md) links to the updated files.
