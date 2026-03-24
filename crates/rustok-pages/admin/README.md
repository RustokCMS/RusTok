# rustok-pages-admin

## Purpose

`rustok-pages-admin` publishes the Leptos admin root page for the `rustok-pages` module.

## Responsibilities

- Export the module-owned `PagesAdmin` root component for `apps/admin`.
- Keep pages-specific admin UI inside the module boundary instead of `apps/admin`.
- Act as the canonical working admin vertical slice for module-owned page CRUD.

## Interactions

- Used by `apps/admin` through manifest-driven generated wiring.
- Uses the pages module GraphQL contract for list/create/edit/update/publish/delete flows.
- Follows the generic host route contract `/modules/:module_slug`.

## Entry points

- `PagesAdmin`
