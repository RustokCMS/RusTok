# rustok-inventory-admin

Leptos admin UI package for the `rustok-inventory` module.

## Responsibilities

- Exposes the inventory operations admin root view used by `apps/admin`.
- Keeps inventory visibility and stock-health UX inside the inventory-owned package.
- Participates in manifest-driven admin composition through `rustok-module.toml`.
- Uses the existing `rustok-commerce` GraphQL product contract for current read-side inventory visibility while dedicated inventory transport is still being split out.
- Ships package-owned `admin/locales/en.json` and `admin/locales/ru.json` bundles declared through `[provides.admin_ui.i18n]`.

## Entry Points

- `InventoryAdmin` - root admin view rendered from the host admin registry.

## Interactions

- Consumed by `apps/admin` via manifest-driven `build.rs` code generation.
- Reads inventory-related product and variant fields from the current `rustok-commerce` GraphQL contract in parallel with the ongoing ecommerce UI split.
- Reads the effective UI locale from `UiRouteContext.locale`; inventory detail cards resolve localized product copy against that host-owned locale and only fall back when that locale is missing.

## Documentation

- See [platform docs](../../../docs/index.md).
