# rustok-content

## Purpose

`rustok-content` owns the core CMS domain for RusToK: nodes, posts, media, comments,
categories, and tags.

## Responsibilities

- Provide `ContentModule` metadata for the runtime registry.
- Own content entities, services, orchestration, and migrations.
- Own content GraphQL and REST transport adapters for module-facing APIs.
- Publish module-owned Leptos UI packages in `admin/` and `storefront/` for host composition.
- Publish the typed RBAC surface for content resources such as `nodes:*`, `posts:*`,
  `media:*`, `comments:*`, `categories:*`, and `tags:*`.

## Interactions

- Depends on `rustok-core` for permissions, events, and `SecurityContext`.
- Depends on `rustok-api` for shared tenant/auth/request and GraphQL helper contracts.
- Exposes its own GraphQL and REST adapters; `apps/server` now acts only as a composition root
  and re-export shim for content transport entry points.
- Used as a storage/orchestration dependency by `rustok-blog`, `rustok-forum`, and `rustok-pages`.
- Declares permissions via `rustok-core::Permission`.
- REST and GraphQL mutations enforce permissions from `AuthContext.permissions`, then pass a
  permission-aware `SecurityContext` into content services.
- `apps/admin` consumes `rustok-content-admin` and `apps/storefront` consumes
  `rustok-content-storefront` through manifest-driven `build.rs` code generation.

## Entry points

- `ContentModule`
- `NodeService`
- `ContentOrchestrationService`
- `CategoryService`
- `TagService`
- `graphql::ContentQuery`
- `graphql::ContentMutation`
- `controllers::routes`
- `admin::ContentAdmin` (publishable Leptos package)
- `storefront::ContentView` (publishable Leptos package)
- content DTO and entity re-exports
