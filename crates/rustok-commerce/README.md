# rustok-commerce

## Purpose

`rustok-commerce` is the `Ecommerce` umbrella/root module for RusToK's commerce family.

## Responsibilities

- Provide `CommerceModule` metadata for the runtime registry.
- Serve as the umbrella entry point for the ecommerce family.
- Preserve the legacy GraphQL and REST transport surface during the transition.
- Orchestrate submodules of the ecommerce family through the compatibility layer.
- Re-export the shared DTO/entity/error surface from `rustok-commerce-foundation`.
- Re-export `CartService`, `CustomerService`, `CatalogService`, `PricingService`, `InventoryService`, `OrderService`, `PaymentService`, and `FulfillmentService` from the split modules.
- Keep legacy commerce-owned state-machine and leftover migrations not yet moved to new modules.
- Publish a module-owned Leptos admin UI package in `admin/` for host composition.
- Publish a module-owned Leptos storefront UI package in `storefront/` for host composition.
- Publish the typed RBAC surface for commerce resources.

## Interactions

- Depends on `rustok-core` for module contracts and permission vocabulary.
- Depends on `rustok-commerce-foundation` for shared DTOs, entities, search helpers, and errors.
- Depends on `rustok-cart`, `rustok-customer`, `rustok-product`, `rustok-pricing`, `rustok-inventory`, `rustok-order`,
  `rustok-payment`, and `rustok-fulfillment` as the default cart, customer, product, pricing, inventory, order,
  payment, and fulfillment submodules of the ecommerce family.
- Depends on `rustok-api` for shared auth/tenant/request GraphQL+HTTP adapter contracts.
- Depends on `rustok-outbox` and `rustok-events` for transactional domain-event publishing.
- Used by `apps/server` through thin GraphQL/REST shims and route composition.
- `apps/admin` consumes `rustok-commerce-admin` through manifest-driven `build.rs` code generation, with a module-owned catalog control room mounted under `/modules/commerce`.
- `apps/storefront` consumes `rustok-commerce-storefront` through manifest-driven `build.rs` code generation, with a public catalog surface mounted under `/modules/commerce`.
- Declares permissions via `rustok-core::Permission` for `products`, `orders`, `customers`,
  `payments`, `fulfillments`, `inventory`, and `discounts`.
- Transport adapters validate permissions against `AuthContext.permissions`, then invoke
  commerce services or direct tenant-scoped SeaORM reads where the module still owns the
  read-model assembly.

## Entry points

- `CommerceModule`
- `CartService`
- `CustomerService`
- `CatalogService`
- `PricingService`
- `InventoryService`
- `OrderService`
- `PaymentService`
- `FulfillmentService`
- `graphql::CommerceQuery`
- `graphql::CommerceMutation`
- `controllers::routes`
- `admin::CommerceAdmin` (publishable Leptos package)
- `storefront::CommerceView` (publishable Leptos package)
- commerce DTO and state-machine re-exports

See also `docs/README.md`.
