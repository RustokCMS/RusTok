# rustok-search-storefront

Leptos storefront UI package for the `rustok-search` module.

## Responsibilities

- Exposes the search storefront root view used by `apps/storefront`.
- Keeps search-specific storefront UX inside the module package.
- Participates in the manifest-driven UI composition path through `rustok-module.toml`.
- Provides the baseline route/slot scaffold for query input, suggestions, filters, and results.

## Entry Points

- `SearchView` — root storefront view rendered from the host storefront slot registry.

## Interactions

- Consumed by `apps/storefront` via manifest-driven `build.rs` code generation.
- Uses the shared `UiRouteContext` to read query-string state without leaking host-specific routing details.
- Will remain aligned with the future Next storefront package on the same API/query model.

## Documentation

- See [platform docs](../../../docs/index.md).
