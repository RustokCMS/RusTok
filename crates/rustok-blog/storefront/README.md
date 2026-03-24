# rustok-blog-storefront

Leptos storefront UI package for the `rustok-blog` module.

## Responsibilities

- Exposes the blog storefront root view used by `apps/storefront`.
- Keeps blog-specific storefront UI inside the module package.
- Participates in the manifest-driven UI composition path through `rustok-module.toml`.
- Owns the standard GraphQL read-path for published posts and selected `?slug=` rendering.

## Entry Points

- `BlogView` — root storefront view rendered from the host storefront slot registry.

## Interactions

- Consumed by `apps/storefront` via manifest-driven `build.rs` code generation.
- Uses the `rustok-blog` GraphQL contract plus shared host libraries such as `UiRouteContext`.
- Should remain compatible with the host storefront slot and generic module page contract.

## Documentation

- See [platform docs](../../../docs/index.md).
