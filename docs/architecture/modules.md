# Module map

RusToK is composed of domain modules under `crates/` and application wiring under `apps/`.

## Module registry and manifests

- [`modules.toml`](../../modules.toml)
- [Module overview](../modules/overview.md)
- [Module & application registry](../modules/registry.md)
- [Module manifest](../modules/manifest.md)

## Composition

- `apps/server` wires module routes, GraphQL schema, and background workers.
- `apps/admin` and `apps/storefront` are Leptos frontends.
- `apps/next-admin` and `apps/next-frontend` are Next.js frontends.

## Additional references

- [Architecture overview](./overview.md)
- [Routing policy](./routing.md)
- [Events and outbox](./events.md)
- [Improvement Recommendations](./improvement-recommendations.md)
