# rustok-content-storefront

Leptos storefront UI package for the `rustok-content` module.

## Responsibilities

- Exposes the content storefront root view used by `apps/storefront`.
- Keeps content-specific storefront UI inside the module package.
- Participates in the manifest-driven UI composition path through `rustok-module.toml`.
- Owns the GraphQL read-path for published nodes and selected `?id=` rendering.

## Entry Points

- `ContentView` - root storefront view rendered from the host storefront slot registry.

## Interactions

- Consumed by `apps/storefront` via manifest-driven `build.rs` code generation.
- Uses the `rustok-content` GraphQL contract plus shared host libraries such as `UiRouteContext`.
- Resolves `currentTenant` before node lookups because the content GraphQL surface still expects `tenantId` on node queries.

## Documentation

- See [platform docs](../../../docs/index.md).
