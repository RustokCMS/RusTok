# rustok-forum

## Purpose

`rustok-forum` owns the forum domain built on top of the content module.

## Responsibilities

- Provide `ForumModule` metadata for the runtime registry.
- Own forum categories, topics, replies, and moderation workflows.
- Own forum GraphQL and REST transport adapters alongside the domain services.
- Publish the typed RBAC surface for `forum_categories:*`, `forum_topics:*`,
  and `forum_replies:*`.

## Interactions

- Depends on `rustok-content` for shared content storage and orchestration primitives.
- Depends on `rustok-core` for module contracts, permissions, and `SecurityContext`.
- Depends on `rustok-api` for shared auth/tenant/request GraphQL+HTTP adapter contracts.
- Used by `apps/server` through thin GraphQL/REST shims and route composition.
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
