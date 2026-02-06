# Admin Phase 3 architecture (Server + Leptos + Next.js)

This document describes the **implemented architecture** for Phase 3 admin auth/security
flows and the contracts that both admin frontends should follow.

Related docs:
- Scope: `docs/admin-auth-phase3.md`
- Progress/gaps: `docs/admin-phase3-gap-analysis.md`

## 1) Runtime layers

## 1.1 Backend (`apps/server`)

Phase 3 logic lives in `apps/server/src/controllers/auth.rs` and `apps/server/src/auth.rs`.

Core responsibilities:
- Credentials login/register and token issuance.
- Password reset token generation/validation.
- Profile update.
- Password change.
- Session listing/history/revoke-all.

## 1.2 Leptos admin (`apps/admin`)

Leptos app calls server REST endpoints through shared helpers in `apps/admin/src/api/mod.rs`.
Auth state is kept in `AuthContext` with persisted:
- `rustok-admin-token`
- `rustok-admin-user`
- `rustok-admin-tenant`

This keeps all protected calls tenant-scoped without requiring repeated tenant input.

## 1.3 Next admin (`apps/next-admin`)

Next app uses locale routes (`/[locale]/*`) and cookie-based auth context:
- `rustok-admin-token`
- `rustok-admin-tenant`

Client views use direct fetch calls to `/api/auth/*`. A shared helper
`src/lib/client-auth.ts` centralizes cookie parsing.

---

## 2) HTTP contract map (Phase 3)

All endpoints are under `/api/auth`.

### Public endpoints
- `POST /login`
- `POST /register`
- `POST /reset/request`
- `POST /reset/confirm`

### Protected endpoints (Bearer + tenant header)
- `GET /me`
- `POST /profile`
- `POST /change-password`
- `GET /sessions`
- `GET /history`
- `POST /sessions/revoke-all`

Tenant scoping is via `X-Tenant-Slug` + token tenant validation.

---

## 3) Security model

## 3.1 Access token claims

Access token includes user, tenant, role, and `session_id`.
`session_id` is propagated into `CurrentUser` extractor so protected handlers
can preserve current session when revoking others.

## 3.2 Password reset token claims

Reset flow uses a dedicated JWT claim model (`PasswordResetClaims`) with:
- `tenant_id`
- `sub` (email)
- `purpose=password_reset`
- expiration (`DEFAULT_RESET_TOKEN_TTL_SECS`)

Confirm endpoint validates signature, expiry, purpose, and tenant match.

## 3.3 Session lifecycle

- Login/Register create refresh session records.
- `change-password` revokes all sessions except current.
- `sessions/revoke-all` revokes all sessions except current.
- `sessions` returns active sessions.
- `history` returns recent session activity entries.

---

## 4) Frontend behavior contract

To keep parity between Leptos and Next:

1. **Tenant propagation**
   - On successful login/register, persist tenant.
   - Use persisted tenant for all protected requests.

2. **Error mapping**
   - `401` -> `errors.auth.unauthorized` or `errors.auth.invalid_credentials` (login).
   - non-2xx -> `errors.http`.
   - network exceptions -> `errors.network`.

3. **State model**
   - Each page exposes explicit `status` and `error` states.
   - Reset request can surface token preview in demo mode.

4. **Locale model**
   - Next uses `next-intl` dictionaries (`messages/en.json`, `messages/ru.json`).
   - Leptos uses `translate(...)` dictionaries under providers.

---

## 5) Current implementation status

Implemented end-to-end:
- `/login`
- `/register`
- `/reset`
- `/profile`
- `/security` (change password + sessions + history + revoke-all)

Still open in Phase 3 scope:
- invite acceptance backend + UI flow
- email verification/resend backend + UI flow
- auth audit event surfacing in dedicated admin feed

---

## 6) Sequence snapshots

## 6.1 Register

1. UI sends `POST /api/auth/register` + `X-Tenant-Slug`.
2. Server creates user and initial session.
3. Server returns access token + user info.
4. Frontend persists token + tenant + user.

## 6.2 Reset

1. UI sends `POST /api/auth/reset/request`.
2. Server returns generic success (and optional token in demo mode).
3. UI sends `POST /api/auth/reset/confirm` with token + new password.
4. Server validates reset token claims and updates password hash.

## 6.3 Security revoke-all

1. UI sends `POST /api/auth/sessions/revoke-all` with Bearer token.
2. Server resolves `CurrentUser` with current `session_id`.
3. Server revokes all other sessions for user/tenant.
4. UI refreshes `GET /api/auth/sessions`.
