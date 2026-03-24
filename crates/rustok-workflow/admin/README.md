# rustok-workflow-admin

Leptos admin UI package for the `rustok-workflow` module.

## Responsibilities

- Exposes the workflow admin root view used by `apps/admin`.
- Keeps workflow-specific admin UI inside the module boundary instead of `apps/admin`.
- Participates in manifest-driven host composition through `rustok-module.toml`.

## Entry Points

- `WorkflowAdmin` — root admin page component for the module.
- `rustok-module.toml [provides.admin_ui]` advertises `leptos_crate`, `route_segment`, `nav_label`, and manifest-driven nested subpages such as `templates`.

## Interactions

- Consumed by `apps/admin` via manifest-driven `build.rs` code generation.
- Mounted by the Leptos admin host under `/modules/workflow` and `/modules/workflow/templates` through the generic module page route contract.
- Uses shared auth and GraphQL transport hooks exposed to admin-side Leptos packages.
- Uses shared `UiRouteContext` to branch between overview and templates without `apps/admin` knowing workflow-specific routes.
- Temporarily links workflow rows back to the legacy `/workflows/:id` detail flow until the full editor also moves behind the module-owned contract.

## Documentation

- See [platform docs](../../../docs/index.md).
