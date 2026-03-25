# rustok-forum-storefront

Leptos storefront UI package for the `rustok-forum` module.

## Responsibilities

- Exposes the forum storefront root view used by `apps/storefront`.
- Keeps forum-specific storefront UX inside the module package.
- Participates in the manifest-driven UI composition path through `rustok-module.toml`.
- Owns the public GraphQL read-path for category feeds, selected threads, and reply rendering.
- Presents the module as a NodeBB-inspired public discussion surface.

## Entry Points

- `ForumView` - root storefront view rendered from the host storefront slot registry.

## Interactions

- Consumed by `apps/storefront` via manifest-driven `build.rs` code generation.
- Uses the `rustok-forum` GraphQL storefront contract plus shared host libraries such as `UiRouteContext`.
- Should remain compatible with the host storefront slot and generic module page contract.

## Documentation

- See [platform docs](../../../docs/index.md).
