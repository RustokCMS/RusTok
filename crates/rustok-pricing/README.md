# rustok-pricing

## Purpose

`rustok-pricing` is the default pricing submodule of the `Ecommerce` family.

## Responsibilities

- Pricing service, price-related migrations, and pricing runtime metadata.
- Keep the decimal money contract working on the current runtime while preserving
  compatibility with legacy `prices.amount` / `compare_at_amount` storage.

## Interactions

- Depends on `rustok-commerce-foundation` for shared commerce DTOs/entities/errors.
- Depends on `rustok-product` data model through variant references.
- Used by `rustok-commerce` as the umbrella/root module of the ecommerce family.

## Entry points

- `PricingModule`
- `PricingService`

See also `docs/README.md`.
