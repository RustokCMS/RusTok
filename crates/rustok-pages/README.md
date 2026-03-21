# rustok-pages

## Purpose

`rustok-pages` owns static pages, blocks, and menus for RusToK.

## Responsibilities

- Provide `PagesModule` metadata for the runtime registry.
- Own page, block, and menu services layered on top of content storage.
- Own the Pages GraphQL and REST adapters exported from the module crate.
- Publish the typed RBAC surface for `pages:*` and the node-based block helpers it needs.

## Interactions

- Depends on `rustok-content` for shared node storage and content helpers.
- Depends on `rustok-core` for module contracts, permissions, and `SecurityContext`.
- Depends on `rustok-api` for shared tenant/auth/request/GraphQL helper contracts.
- Used by `apps/server` as a composition-root dependency; server now re-exports module-owned pages GraphQL and REST entry points.
- Declares permissions via `rustok-core::Permission`.
- Module adapters enforce `pages:*` permissions from `AuthContext.permissions` and pass a
  permission-aware `SecurityContext` into page services.

## Entry points

- `PagesModule`
- `PageService`
- `BlockService`
- `MenuService`
- `graphql::PagesQuery`
- `graphql::PagesMutation`
- `controllers::routes`
