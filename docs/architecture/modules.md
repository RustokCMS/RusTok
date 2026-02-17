# Module map

RusToK is composed of domain modules under `crates/` and application wiring under `apps/`.

## Module registry

- [`modules.toml`](../../modules.toml)
- [`docs/modules/modules.md`](../modules/modules.md)
- [`docs/modules/MODULE_MATRIX.md`](../modules/MODULE_MATRIX.md)

## Composition

- `apps/server` wires module routes, GraphQL schema, and background workers.
- `apps/admin` and `apps/storefront` consume module APIs and shared UI packages.

## Additional references

- [Module manifest](../modules/module-manifest.md)
- [Module registry](../modules/module-registry.md)
