# alloy

## Purpose

`alloy` owns the GraphQL and REST transport adapters for the module-agnostic Alloy
capability.

## Responsibilities

- Expose Alloy GraphQL adapters for script CRUD and manual execution.
- Expose Alloy REST entry points as a thin transport shell over `alloy-scripting`.
- Keep `apps/server` as a composition root while isolating web-facing Alloy contracts from the
  scripting runtime crate.

## Interactions

- Depends on `alloy-scripting` for the Rhai runtime, storage, execution log, and REST handler core.
- Depends on `rustok-api` for shared auth/tenant and GraphQL helper contracts.
- Depends on `rustok-core` for the typed `scripts:*` permission surface used by GraphQL auth guards.
- Serves as the server-facing transport layer for the Alloy capability while Alloy itself remains
  outside the tenant module registry.
- `apps/server` now mounts `alloy` as a shim/composition-root layer instead of owning Alloy
  GraphQL code directly.

## Entry points

- `graphql::AlloyQuery`
- `graphql::AlloyMutation`
- `graphql::AlloyState`
- `controllers::router`

