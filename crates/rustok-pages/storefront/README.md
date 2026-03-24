# rustok-pages-storefront

## Purpose

`rustok-pages-storefront` publishes the Leptos storefront root view for the `rustok-pages` module.

## Responsibilities

- Export the module-owned `PagesView` root component for `apps/storefront`.
- Keep pages-specific storefront rendering inside the module boundary.
- Act as the canonical working storefront read-path for published pages.

## Interactions

- Used by `apps/storefront` through manifest-driven generated wiring.
- Uses the pages module GraphQL read contract plus shared `UiRouteContext` to render a slug-selected page and a small page directory.
- Follows the generic storefront host contract: slots plus `/modules/:route_segment`.

## Entry points

- `PagesView`
