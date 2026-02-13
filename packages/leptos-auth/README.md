# Leptos Auth

Specialized internal library that standardizes authentication flows, storage keys,
error mapping, and UX contracts across **both admin and storefront frontends**:
- `apps/admin` (Leptos CSR)
- `apps/storefront` (Leptos)
- `apps/next-frontend` (Next.js)

This kit is **intentionally narrow and opinionated** for RusTok: it mirrors the
current `/api/auth/*` REST contract and the Phase 3 admin architecture decisions.
It is **not** a general-purpose auth framework.

---

## Why this exists

We need identical auth behavior in two different UI stacks. The kit provides
one shared contract for:
- storage keys (token, tenant, user)
- error mapping and status handling
- core auth data shapes
- helper APIs for each runtime (Rust/Leptos and TypeScript/Next)

This keeps the UI minimal, avoids ad-hoc divergence, and reduces the amount of
custom code on each page.

---

## Scope (what this kit covers)

**Frontend contract only** (no backend logic):
- Login/register/reset/profile/security flows.
- Tenant propagation for all protected calls.
- Error mapping for `401` vs other HTTP statuses.
- Persistent storage keys for admin auth context.
- Minimal helper API for Next.js (cookies) and Leptos (localStorage).

---

## Non-goals

- SSO/OIDC/SAML, passwordless, 2FA/TOTP (explicitly out of Phase 3 scope).
- Replacing the existing REST endpoints or server auth logic.
- Acting as a universal auth framework for non-RusTok UIs.

---

## API contract summary

### REST endpoints
All endpoints live under `/api/auth`:

**Public**
- `POST /login`
- `POST /register`
- `POST /reset/request`
- `POST /reset/confirm`

**Protected** (Bearer token + tenant header)
- `GET /me`
- `POST /profile`
- `POST /change-password`
- `GET /sessions`
- `GET /history`
- `POST /sessions/revoke-all`

### Auth storage keys
These keys are **fixed** and used in both admin and storefront apps:
- `rustok-admin-token`
- `rustok-admin-tenant`
- `rustok-admin-user`

---

## Error mapping contract

- `401` in **login** flow → `invalid_credentials`
- `401` in other protected flows → `unauthorized`
- any non-2xx → `http` (with status)
- network/transport failures → `network`

---

## Runtime implementations

### 1) Leptos (Rust)
Rust helpers live in `crates/leptos-auth` and are meant to be used in
`apps/admin` or `apps/storefront` (or other Rust-based frontends).

Exports:
- `AuthUser`, `AuthSession`
- `AuthError` with `from_status(...)`
- storage key constants

### 2) Next.js (TypeScript)
TypeScript helpers live in `packages/leptos-auth/next` and are meant to be used in
`apps/next-frontend` (or other React-based frontends).

Exports:
- `AuthUser`, `AuthSession`
- `mapAuthError(...)`
- cookie helpers: `getCookieValue(...)`, `getClientAuth(...)`
- storage key constants

To consume in Next.js, add a path alias or import it via a relative path from the
monorepo (e.g. by wiring `tsconfig` paths to `packages/leptos-auth/next`).

---

## Usage (examples)

### Leptos
```rust
use leptos_auth::{AuthError, AuthSession, ADMIN_TENANT_KEY};

let error = AuthError::from_status(401, true);
```

### Next.js
```ts
import { getClientAuth, mapAuthError } from "@/lib/leptos-auth";

const { token, tenant } = getClientAuth();
const error = mapAuthError(401, true);
```

---

## Extending the kit

When new auth UX is added, extend **the contract first**, then implement in
both runtimes:
1. Define the new error/status/field contract here.
2. Add Rust helper types/functions.
3. Add TypeScript helper types/functions.
4. Only then wire new UI pages.

This keeps parity between Leptos and Next and avoids hidden behavior drift.

---

## Repo layout

```
crates/leptos-auth/   # Rust helper crate for Leptos frontends
packages/leptos-auth/ # TS helper module for Next.js frontends
```

---

## Status

**Minimal baseline** for Phase 3 admin auth. Designed to grow as the template
integration lands.

This is an alpha version and requires clarification. Be careful, there may be errors in the text. So that no one thinks that this is an immutable rule.
