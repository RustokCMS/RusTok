# Module Registry (Step 7)

This document describes how the module registry works in the current codebase
and how per-tenant enable/disable is enforced.

## Overview

RusToK compiles modules into the binary but keeps the per-tenant state in the
`tenant_modules` table. The registry exposes the set of compiled modules and
their metadata (slug, name, description, version, dependencies). Per-tenant
status is resolved at runtime.

To make the module set dynamic (WordPress/NodeBB-style), the build uses a
module manifest that defines which crates are included in the build. A rebuild
produces a new binary and therefore a new registry set.

Key pieces:

- **Core module contract**: `crates/rustok-core/src/module.rs` defines
  `RusToKModule` and lifecycle hooks.
- **Module registry**: `crates/rustok-core/src/registry.rs` stores compiled
  module implementations.
- **Server registry bootstrap**: `apps/server/src/modules/mod.rs` registers
  built-in modules and is attached to the app in `apps/server/src/app.rs`.
- **Module manifest**: `docs/modules/module-manifest.md` describes the build
  manifest used to select which modules are compiled into the binary.
- **GraphQL**: `module_registry` query combines registry metadata with per-tenant
  state, and `toggle_module` updates the state and calls lifecycle hooks.
- **Route enforcement**: `apps/server/src/guards/module.rs` provides
  `RequireModule<SLUG>` for controllers to block access when a module is
  disabled.

## Module lifecycle

`toggle_module` delegates orchestration to `ModuleLifecycleService`
(`apps/server/src/services/module_lifecycle.rs`). The service performs:

1. Dependency validation and dependent-module checks.
2. Transactional state persistence in `tenant_modules`.
3. Hook invocation (`on_enable` / `on_disable`) only when state actually changed
   (idempotent behavior for repeated toggle requests with the same target state).
4. Compensating rollback to previous state if hook execution fails.

This keeps GraphQL resolver logic thin and ensures tenant module state remains
consistent even when hooks return an error.

Hooks receive `ModuleContext`, which includes the database connection, tenant
ID, and the module settings payload.

## Per-tenant toggles

The `tenant_modules` table is the source of truth for enabled/disabled flags.
The server queries this table to determine module status for the active tenant.

## How to enforce module access

Controllers that belong to a specific module should add the guard to reject
requests when the module is disabled:

```rust
use crate::guards::module::RequireModule;

async fn handler(
    _module: RequireModule<"commerce">,
    // ...
) -> impl IntoResponse {
    // ...
}
```

## GraphQL

Example queries/mutations:

```graphql
query ModuleRegistry {
  moduleRegistry {
    moduleSlug
    name
    description
    version
    dependencies
    enabled
  }
}

mutation ToggleModule {
  toggleModule(moduleSlug: "blog", enabled: true) {
    moduleSlug
    enabled
  }
}
```

This is an alpha version and requires clarification. Be careful, there may be errors in the text. So that no one thinks that this is an immutable rule.
