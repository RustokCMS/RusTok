# rustok-forum

## Purpose

`rustok-forum` owns the forum domain with forum-owned persistence.

## Responsibilities

- Provide `ForumModule` metadata for the runtime registry.
- Own forum categories, topics, replies, and moderation workflows.
- Apply module-owned reply lifecycle rules, including pending replies for moderated categories and approved-only public storefront reads.
- Own forum storage tables for categories, topics, translations, replies, and channel access.
- Own forum GraphQL and REST transport adapters alongside the domain services.
- Publish a module-owned Leptos admin UI package in `admin/` for host composition.
- Publish a module-owned Leptos storefront UI package in `storefront/` for host composition.
- Publish the typed RBAC surface for `forum_categories:*`, `forum_topics:*`,
  and `forum_replies:*`.

## Interactions

- Depends on `rustok-content` for shared rich-text, locale, and future orchestration helpers.
- Depends on `rustok-core` for module contracts, permissions, and `SecurityContext`.
- Depends on `rustok-api` for shared auth/tenant/request GraphQL+HTTP adapter contracts.
- Used by `apps/server` through thin GraphQL/REST shims and route composition.
- `apps/admin` consumes `rustok-forum-admin` through manifest-driven `build.rs` code generation, with a NodeBB-inspired moderation workspace mounted under `/modules/forum`.
- `apps/storefront` consumes `rustok-forum-storefront` through manifest-driven `build.rs` code generation, with a public NodeBB-inspired discussion feed mounted under `/modules/forum`.
- Declares permissions via `rustok-core::Permission`.
- Transport adapters validate forum permissions against `AuthContext.permissions`, then pass
  a permission-aware `SecurityContext` into forum services.

## Entry points

- `ForumModule`
- `TopicService`
- `ReplyService`
- `CategoryService`
- `ModerationService`
- `graphql::ForumQuery`
- `graphql::ForumMutation`
- `controllers::routes`
- `admin::ForumAdmin` (publishable Leptos package)
- `storefront::ForumView` (publishable Leptos package)
