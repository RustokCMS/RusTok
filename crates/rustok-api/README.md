# rustok-api

## Purpose
`rustok-api` is the shared web/API adapter layer for RusToK. It hosts reusable request, tenant, auth, and GraphQL-facing contracts that should be available to `apps/server` and, over time, to module crates that expose GraphQL or HTTP adapters.

## Responsibilities
- Provide reusable tenant and auth request context types.
- Provide GraphQL helper types and error helpers shared across modules.
- Provide request-level locale and tenant resolution primitives that do not belong in domain crates.
- Keep web-framework-oriented dependencies out of `rustok-core` while still allowing modular reuse.

## Interactions
- Used by `apps/server` as the current composition root.
- Intended to be used by module crates such as `rustok-blog`, `rustok-content`, `rustok-commerce`, and others when their GraphQL/REST adapters move out of `apps/server`.
- Depends on `rustok-core` for core security and permission primitives.
- Depends on `rustok-tenant` and `rustok-content` for tenant-module enablement checks and locale defaults.

## Entry Points
- `src/lib.rs`
- `src/context/`
- `src/request.rs`
- `src/graphql/`

## Documentation
- Local docs: `./docs/`
- Platform docs: `../../docs/`
