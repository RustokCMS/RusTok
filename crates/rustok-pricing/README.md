# rustok-pricing

## Purpose

`rustok-pricing` is the default pricing submodule of the `Ecommerce` family.

## Responsibilities

- Own the pricing service, price-related migrations, and pricing runtime metadata.
- Keep the decimal money contract working on the current runtime while preserving
  compatibility with legacy `prices.amount` and `compare_at_amount` storage.
- Start Pricing 2.0 with a typed price-resolution foundation over
  `currency_code + optional region_id + optional quantity`, and now allow
  explicit `price_list_id` overlays when the requested list is active for the
  current tenant/time window, plus a normalized `discount_percent` on resolved
  sale prices; the resolver now also consumes host-provided `channel_id` /
  `channel_slug` and supports channel-scoped base rows and price lists without
  making `rustok-pricing` the owner of channel identity.
- Expose a typed percentage-adjustment preview/apply contract inside
  `PricingService`, while keeping legacy `apply_discount` as a compatibility
  wrapper over the new adjustment path; the typed path now supports both the
  canonical base row and active `price_list` override rows.
- Expose active tenant-scoped price lists as a pricing-owned read contract so
  admin and storefront selectors do not depend on raw UUID-only entry, and now
  carry typed rule metadata plus optional channel scope for fallback sale
  semantics.
- Expose first-class `price_list` percentage rules so an active list can derive
  effective sale prices from base rows even without explicit override rows.
- Provide a module-owned Leptos admin UI package in `admin/` for pricing visibility,
  sale markers, currency-coverage inspection, effective price inspection, and
  operator updates for base rows and active price-list override rows on variant prices,
  including quantity tiers and typed percentage-discount adjustments on the
  canonical base row or the selected active price-list override row.
- Provide a module-owned Leptos storefront UI package in `storefront/` for
  public pricing discovery, currency coverage, sale-marker visibility, and
  effective price previews.

## Interactions

- Depends on `rustok-commerce-foundation` for shared commerce DTOs, entities, and errors.
- Depends on `rustok-product` data model through variant references.
- Used by `rustok-commerce` as the umbrella/root module of the ecommerce family.
- `apps/admin` consumes `rustok-pricing-admin` through manifest-driven composition,
  and now gets native module-owned base-price and active price-list override write
  actions there, plus base-row percentage-discount preview/apply and selected
  price-list rule editing, while
  promotion/rule-aware pricing transport is still being split from the umbrella
  commerce surface.
- `apps/storefront` consumes `rustok-pricing-storefront` through manifest-driven
  composition for a public pricing atlas route.

## Entry points

- `PricingModule`
- `PricingService`
- `PriceResolutionContext`
- `ResolvedPrice`
- `rustok-pricing-admin`
- `PricingView`

See also `docs/README.md`.
