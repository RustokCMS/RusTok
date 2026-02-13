# Technical parity plan (Leptos-first)

This document tracks **technical parity** with Leptos as the source of truth for admin and storefront, plus optional Next.js storefront alignment.
It focuses on shared integration points like GraphQL, routing, SEO, auth, and error handling. The goal is
feature parity **in behavior and integration**, independent of UI design.

## Scope

- Admin: `apps/admin` (Leptos CSR).
- Storefront: `apps/storefront` (Leptos SSR) + `apps/next-frontend` (Next.js).

## Parity goals

- Same API capabilities and data contracts in both stacks.
- Same navigation flows, route guards, and URL structures.
- Same SEO strategy and metadata coverage where applicable.
- Same auth/session behavior and error handling.
- Accept small behavioral differences when libraries are not 1:1, but document them clearly.
- Keep the documentation current as Leptos libraries catch up with Next.js features.

## Integration parity matrix

| Area | Next.js (admin) | Leptos (admin) | Next.js (storefront) | Leptos (storefront) | Notes |
| --- | --- | --- | --- | --- | --- |
| GraphQL integration | ✅ | ⚠️ | ✅ | ⚠️ | Align query/mutation names and filtering semantics.
| Routing structure | ✅ | ✅ | ✅ | ⚠️ | Match paths, dynamic segments, and redirects.
| Auth/session | ✅ | ⚠️ | ✅ | ⚠️ | Match token storage, expiry, refresh, and logout.
| Error handling | ✅ | ⚠️ | ✅ | ⚠️ | Standardize error codes → UI states.
| i18n | ✅ | ⚠️ | ✅ | ⚠️ | Same locale list, fallback order, and URL strategy.
| Telemetry | ✅ | ⚠️ | ✅ | ⚠️ | Same events, context payload, and sampling.
| Feature flags | ✅ | ⚠️ | ✅ | ⚠️ | Same flag sources and gating logic.
| Forms/validation | ✅ | ⚠️ | ✅ | ⚠️ | Same constraints, messages, and server mapping.
| SSR/CSR strategy | ✅ | ⚠️ | ✅ | ⚠️ | Align hydration and caching strategy.
| SEO | ✅ | ⚠️ | ✅ | ⚠️ | Same metadata, structured data, and robots rules.
| Auth kit helpers (`leptos-auth`) | — | ✅ | ✅ | ✅ | Use in `apps/admin`, `apps/storefront`, `apps/next-frontend`.
| GraphQL kit helpers (`leptos-graphql`) | ✅ | ✅ | ✅ | ✅ | Standardize `/api/graphql` headers and request shapes across all apps.
| Form kit helpers (`leptos-hook-form`) | ✅ | ✅ | ✅ | ✅ | Standardize form state + errors across admin/storefront.
| Validation kit helpers (`leptos-zod`) | ✅ | ✅ | ✅ | ✅ | Standardize zod-style validation errors across stacks.
| Table kit helpers (`leptos-table`) | ✅ | ✅ | ✅ | ✅ | Standardize pagination/sort/filter state across stacks.
| Store kit helpers (`leptos-zustand`) | ✅ | ✅ | ✅ | ✅ | Standardize store snapshot/update shapes across stacks.

Legend: ✅ implemented, ⚠️ pending, ❌ not planned.

## Library parity notes

Leptos ecosystem libraries are younger and may lag behind the Next.js equivalents. We accept small
differences as long as they are **explicitly documented** and tracked.

When a feature is missing on the Leptos side:

1. Implement a **custom workaround** in code.
2. Add a code comment noting that this is a temporary custom implementation and should be replaced
   with a library-provided solution when available.
3. Update this document with the gap and a migration note.

### Current gaps / blockers

| Area | Gap | Impact | Mitigation | Migration trigger |
| --- | --- | --- | --- | --- |
| _None_ | _—_ | _—_ | _—_ | _—_ |

### Resolved gaps

| Area | Resolution | Notes |
| --- | --- | --- |
| UI primitives (Leptos) | `leptos-shadcn-ui` builds successfully after dependency alignment (including `lucide-leptos` + `view!` inference). | Keep versions pinned in workspace to avoid regressions. |

### Suggested code annotation format

Use a short, consistent marker near the custom implementation:

```
// PARITY: custom implementation (no Leptos equivalent yet). Replace when available.
```

## Implementation tables

### GraphQL integration

| Item | Contract | Next.js | Leptos | Notes |
| --- | --- | --- | --- | --- |
| Client | GraphQL HTTP | ✅ | ⚠️ | Standardize endpoint + headers.
| Query naming | `users`/`user` | ✅ | ⚠️ | Keep identical filters and sort fields.
| Error mapping | GraphQL errors | ✅ | ⚠️ | Match user-facing messages.

### Routing

| Item | Contract | Next.js | Leptos | Notes |
| --- | --- | --- | --- | --- |
| Admin base | `/admin` | ✅ | ⚠️ | Align base routing and redirect rules.
| Users list | `/users` | ✅ | ⚠️ | Match query params for filters.
| User details | `/users/:id` | ✅ | ⚠️ | Keep ID parsing and breadcrumbs.

### SEO (storefront focus)

| Item | Contract | Next.js | Leptos | Notes |
| --- | --- | --- | --- | --- |
| Title/description | `meta` | ✅ | ⚠️ | Same defaults and overrides.
| OpenGraph | `og:*` | ✅ | ⚠️ | Same image sizing and fallbacks.
| Canonical | `link[rel=canonical]` | ✅ | ⚠️ | Same rules for pagination.
| Robots | `meta[name=robots]` | ✅ | ⚠️ | Same noindex rules for private pages.

## Review process

1. Update this document whenever a new integration is added or changed.
2. Require Leptos implementation first; add Next.js only for storefront parity-sensitive work.
3. Add a snapshot or checklist per release to ensure parity stays intact.
4. Document any parity gaps and the intended migration path.

## Next steps

- Create a shared API contract reference (GraphQL for UI clients) and link it here.
- Define shared route maps for admin and storefront.
- Add a standard error code map and i18n keys list.
- Add a parity gap log (custom implementations + planned migrations).

This is an alpha version and requires clarification. Be careful, there may be errors in the text. So that no one thinks that this is an immutable rule.
