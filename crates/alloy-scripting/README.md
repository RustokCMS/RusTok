# alloy-scripting

## Purpose

`alloy-scripting` owns the Rhai-based scripting runtime for RusToK automation.

## Responsibilities

- Own script storage, execution contracts, and migrations.
- Own the Rhai runtime and REST handler core used by Alloy-facing adapters.
- Publish the typed `scripts:*` RBAC surface.

## Interactions

- Depends on `rustok-core` for permission vocabulary and shared runtime traits.
- Used by `alloy` as the runtime/engine backend for Alloy management/API adapters.
- `apps/server` now composes Alloy through `alloy` transport shims instead of owning
  Alloy transport code directly.
- Integrates with `rustok-mcp` as the module-agnostic Alloy capability surface.
- Used by `rustok-workflow` for script-backed workflow steps without making Alloy part of the
  tenant module graph.
- Declares permissions via `rustok-core::Permission`.
- GraphQL permission checks for `scripts:*` live in `alloy`; the scripting runtime stays
  free of `rustok-api` to avoid dependency cycles through `rustok-core`.

## Entry points

- script storage and execution APIs from `alloy-scripting`
- migrations and runtime configuration helpers

