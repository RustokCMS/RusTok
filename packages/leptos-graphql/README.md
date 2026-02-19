# Leptos GraphQL

Specialized internal helpers for **GraphQL calls** across both admin and storefront
frontends (Leptos + Next.js). This kit standardizes request shapes, headers, and
endpoint constants so we avoid divergent one-off fetch code.

Target apps (keep this in mind during implementation to avoid one-off helpers):
- `apps/admin` (Leptos CSR)
- `apps/storefront` (Leptos SSR)
- `apps/next-frontend` (Next.js)

---

## Why this exists

- Both admin and storefront UIs rely on `/api/graphql`.
- We need consistent tenant propagation and auth headers.
- We want minimal helper APIs that work in both Rust (Leptos) and TS (Next.js).

---

## Contract

- **Endpoint:** `/api/graphql`
- **Tenant header:** `X-Tenant-Slug`
- **Auth header:** `Authorization: Bearer <token>` (when needed)

---

## Runtime implementations

### 1) Leptos (Rust)
Rust helpers live in `crates/leptos-graphql`.

Exports:
- `GraphqlRequest`, `GraphqlResponse`, `GraphqlError`
- `GRAPHQL_ENDPOINT`, `TENANT_HEADER`, `AUTH_HEADER`

### 2) Next.js (TypeScript)
TypeScript helpers live in `packages/leptos-graphql/next`.

Exports:
- `GraphqlRequest`, `GraphqlResponse`, `GraphqlError`
- `fetchGraphql(...)`
- `GRAPHQL_ENDPOINT`, `TENANT_HEADER`, `AUTH_HEADER`

---

## Usage (examples)

### Leptos
```rust
use leptos_graphql::{GraphqlRequest, GRAPHQL_ENDPOINT};

let request = GraphqlRequest::new("query { me { id } }", None::<()>);
```

### Next.js
```ts
import { fetchGraphql } from "@/lib/leptos-graphql";

const response = await fetchGraphql({
  token,
  tenant,
  request: { query: "query { me { id } }" },
});
```

---

## Status

**Minimal baseline** for shared GraphQL fetch behavior. Extend only when a real
use-case appears in admin or storefront UIs.

