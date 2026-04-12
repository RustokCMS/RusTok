# rustok-cart

## Purpose

`rustok-cart` is the default cart submodule of the `Ecommerce` family.

## Responsibilities

- Own the cart write-side schema and line item lifecycle.
- Persist the storefront cart context snapshot across region, locale, customer, and selected shipping intent.
- Persist typed cart adjustments as language-neutral promotion/discount snapshots.
- Keep cart snapshots independent from catalog ownership.
- Support repricing line items via the pricing resolver when quantity or
  storefront context changes, so unit prices do not drift across channel/region
  boundaries.
- Prepare a stable handoff point for later checkout and order placement flows.
- Publish a module-owned storefront cart workspace for cart inspection.

## Interactions

- Depends on `rustok-core` for module contracts.
- Used by `rustok-commerce` as the default cart submodule of the ecommerce family.
- Keeps locale and country snapshots normalized, while letting `rustok-commerce` own cross-module
  store-context resolution against tenant locale policy and regions.
- Keeps adjustment source identity in `source_type/source_id` and never stores promotion display labels
  as cart-owned business data.
- Keeps product and variant references as snapshots so the cart domain does not depend on
  the product module as a lower-level shared layer.
- `apps/storefront` mounts `rustok-cart/storefront` via manifest-driven composition.

## Entry points

- `CartModule`
- `CartService`
- `dto::*`
- `entities::*`
- `CartView`

See also `docs/README.md`.
