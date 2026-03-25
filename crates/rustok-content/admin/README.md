# rustok-content-admin

Leptos admin UI package for the `rustok-content` module.

## Responsibilities

- Exposes the content admin root view used by `apps/admin`.
- Keeps content-specific admin UX inside the module package.
- Participates in the manifest-driven UI composition path through `rustok-module.toml`.
- Owns the GraphQL-backed node CRUD flow: list/create/edit/update/publish/archive/restore/delete.

## Entry Points

- `ContentAdmin` - root admin page component for the module.
- `rustok-module.toml [provides.admin_ui]` advertises `leptos_crate`, `route_segment`, and `nav_label` for host composition.

## Interactions

- Consumed by `apps/admin` via manifest-driven `build.rs` code generation.
- Mounted by the Leptos admin host under `/modules/content` through the generic module page route.
- Uses the `rustok-content` GraphQL contract directly, plus shared Leptos host libraries.
- Resolves `currentTenant` before node CRUD because the content GraphQL surface still expects `tenantId` on its node queries and mutations.

## Documentation

- See [platform docs](../../../docs/index.md).
