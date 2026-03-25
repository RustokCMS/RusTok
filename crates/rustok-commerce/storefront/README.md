# rustok-commerce-storefront

Leptos storefront UI package for the `rustok-commerce` module.

## Responsibilities

- Exposes the commerce storefront root view used by `apps/storefront`.
- Keeps catalog-specific public UX inside the module package.
- Participates in the manifest-driven storefront composition path through `rustok-module.toml`.
- Owns the public GraphQL read-path for published products and selected product detail.

## Entry Points

- `CommerceView` - root storefront view rendered from the host storefront slot registry.

## Interactions

- Consumed by `apps/storefront` via manifest-driven `build.rs` code generation.
- Uses the `rustok-commerce` GraphQL storefront contract plus shared host route context from `rustok-api`.
- Should remain compatible with the host storefront slot and generic module page contract.

## Documentation

- See [platform docs](../../../docs/index.md).
