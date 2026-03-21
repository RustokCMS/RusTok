# rustok-commerce

## Purpose

`rustok-commerce` owns the commerce domain for RusToK: catalog, pricing, inventory,
orders, customers, and discounts.

## Responsibilities

- Provide `CommerceModule` metadata for the runtime registry.
- Own commerce services, entities, state-machine logic, and migrations.
- Own commerce GraphQL and REST transport adapters alongside the domain services.
- Publish the typed RBAC surface for commerce resources.

## Interactions

- Depends on `rustok-core` for module contracts and permission vocabulary.
- Depends on `rustok-api` for shared auth/tenant/request GraphQL+HTTP adapter contracts.
- Depends on `rustok-outbox` and `rustok-events` for transactional domain-event publishing.
- Used by `apps/server` through thin GraphQL/REST shims and route composition.
- Declares permissions via `rustok-core::Permission` for `products`, `orders`, `customers`,
  `inventory`, and `discounts`.
- Transport adapters validate permissions against `AuthContext.permissions`, then invoke
  commerce services or direct tenant-scoped SeaORM reads where the module still owns the
  read-model assembly.

## Entry points

- `CommerceModule`
- `CatalogService`
- `PricingService`
- `InventoryService`
- `graphql::CommerceQuery`
- `graphql::CommerceMutation`
- `controllers::routes`
- commerce DTO and state-machine re-exports
