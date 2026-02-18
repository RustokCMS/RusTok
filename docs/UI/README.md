# UI Documentation Hub

This section documents frontend applications and shared UI integration patterns used in RusToK.

## Current frontend landscape

RusToK currently has four UI applications:

- `apps/next-admin` — primary Next.js admin dashboard.
- `apps/admin` — Leptos admin panel (legacy/parallel implementation).
- `apps/next-frontend` — Next.js storefront.
- `apps/storefront` — Leptos storefront.

For platform-wide app ownership and dependencies, see [`docs/modules/registry.md`](../modules/registry.md).

## Documents in this section

- [GraphQL Architecture](./graphql-architecture.md) — client-side GraphQL conventions.
- [Admin ↔ Server Connection Quickstart](./admin-server-connection-quickstart.md) — backend connection and env setup.
- [Storefront](./storefront.md) — storefront-specific UI notes.
- [Rust UI Component Catalog](./rust-ui-component-catalog.md) — reusable components and crates.

## App-level documentation

- [Next.js Admin README](../../apps/next-admin/README.md)
- [Next.js Admin RBAC Navigation](../../apps/next-admin/docs/nav-rbac.md)
- [Next.js Admin Clerk setup](../../apps/next-admin/docs/clerk_setup.md)
- [Next.js Admin Theming](../../apps/next-admin/docs/themes.md)
- [Leptos Admin docs](../../apps/admin/docs/README.md)
- [Leptos Storefront README](../../apps/storefront/README.md)
- [Next.js Storefront docs](../../apps/next-frontend/docs/README.md)

## Maintenance notes

When frontend architecture, routing, UI contracts, or API integration changes:

1. Update the relevant app-level docs in `apps/*`.
2. Update the corresponding document in `docs/UI/`.
3. Ensure [`docs/index.md`](../index.md) links to the updated files.
