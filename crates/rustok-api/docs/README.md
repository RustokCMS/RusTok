# rustok-api docs

This folder contains the local documentation for `crates/rustok-api`.

## Scope

`rustok-api` is the shared API adapter layer that sits between `rustok-core` and application-specific wiring in `apps/server`. It owns reusable request/auth/tenant/GraphQL primitives, while module-specific resolvers and controllers are migrated into module crates incrementally.
