# Phase 1 (recovered)

## Принципы

- Мы **не клонируем** библиотеки целиком. Вместо этого делаем **минимальные адаптеры/обёртки** и закрываем пробелы **по мере работы** с админками/витриной.
- Приоритет — **готовые библиотеки и интеграции**; самопис — только если нет адекватного аналога.
- Любые отклонения фиксируем в UI‑документах и матрицах паритета.
- Перед разработкой **проверяем установленные библиотеки** и существующие компоненты, чтобы не писать лишний код.

См. базовые источники:
- [UI parity (admin + storefront)](./ui-parity.md)
- [Admin libraries parity](./admin-libraries-parity.md)
- [Admin reuse matrix](./admin-reuse-matrix.md)
- [Tech parity tracker](./tech-parity.md)
- [Storefront overview](./storefront.md)

---

## Phase 1 — чек‑лист (восстановлено по коду)

### Админки (Leptos + Next.js)

Перед разработкой **проверяем установленные библиотеки** и существующие компоненты, чтобы не писать лишний код.

| Работа | Leptos | Next |
| --- | --- | --- |
| Базовый layout и навигационный shell админки. | ✅ | ✅ |
| Dashboard/главная админки. | ✅ | ✅ |
| Страницы аутентификации: login / register / reset. | ✅ | ✅ |
| Страница Security. | ✅ | ✅ |
| Страница Profile. | ✅ | ✅ |
| Users list с фильтрами/поиском и пагинацией (REST + GraphQL запросы). | ✅ | ✅ |
| User details (карточка пользователя). | ✅ | ⬜ |
| Auth‑guard (защита приватных маршрутов). | ✅ | ✅ |
| Базовые UI‑примитивы (PageHeader, кнопки, инпуты) в shadcn‑style. | ✅ | ✅ |
| i18n (RU/EN). | ✅ | ✅ |

### Storefront (Leptos SSR + Next.js)

Перед разработкой **проверяем установленные библиотеки** и существующие компоненты, чтобы не писать лишний код.

| Работа | Leptos | Next |
| --- | --- | --- |
| Landing‑shell (hero + CTA + основной layout). | ✅ | ✅ |
| Блоки контента (карточки/фичи/коллекции). | ✅ | ✅ |
| Блоки маркетинга/инфо (alert/статы/история бренда/подписка). | ✅ | ✅ |
| i18n / локализация витрины. | ✅ | ✅ |
| Tailwind‑стили и базовая тема (DaisyUI/shadcn‑style). | ✅ | ✅ |
| SSR‑сервер + отдача CSS‑бандла. | ✅ | ⬜ |

---

## Phase 2.1 — Users vertical slice (только работы)

| Работа | Leptos | Next |
| --- | --- | --- |
| i18n foundation: ключевые неймспейсы `app/auth/users/errors`. | ⬜ | ⬜ |
| i18n foundation: вынос строк в доменные модули/файлы. | ⬜ | ⬜ |
| i18n foundation: локализация API ошибок (`errors.*`). | ⬜ | ⬜ |
| Auth wiring: `POST /api/auth/login`. | ⬜ | ⬜ |
| Auth wiring: `GET /api/auth/me` (bootstrap). | ⬜ | ⬜ |
| Auth wiring: хранение токена (cookie/localStorage). | ⬜ | ⬜ |
| Users list: GraphQL `users` (pagination). | ⬜ | ⬜ |
| Users list: фильтры и поиск. | ⬜ | ⬜ |
| Users table parity: колонки `email/name/role/status/created_at`. | ⬜ | ⬜ |
| Users detail view: GraphQL `user(id)`. | ⬜ | ⬜ |
| Users CRUD: `createUser` mutation. | ⬜ | ⬜ |
| Users CRUD: `updateUser` mutation. | ⬜ | ⬜ |
| Users CRUD: `disableUser` mutation. | ⬜ | ⬜ |
| Users CRUD: формы, ошибки, тосты. | ⬜ | ⬜ |
| RBAC: права `users.create/users.update/users.manage`. | ⬜ | ⬜ |
| Shared UI/UX: layout/nav parity. | ⬜ | ⬜ |
| Shared UI/UX: breadcrumbs. | ⬜ | ⬜ |
| Shared UI/UX: form patterns. | ⬜ | ⬜ |
| Shared UI/UX: toasts/alerts. | ⬜ | ⬜ |

---

## Phase 3 — Admin Auth & User Security (только работы)

| Работа | Leptos | Next |
| --- | --- | --- |
| Login page: tenant slug + email + password. | ⬜ | ⬜ |
| Login UX: ошибки/валидация, loading/empty states. | ⬜ | ⬜ |
| Language switch + persistence. | ⬜ | ⬜ |
| Password reset: request email. | ⬜ | ⬜ |
| Password reset: token + new password. | ⬜ | ⬜ |
| Password reset: token expiry UI. | ⬜ | ⬜ |
| Email verification: verify + resend action. | ⬜ | ⬜ |
| Registration: sign‑up (email + password + tenant). | ⬜ | ⬜ |
| Registration: optional name + password strength hints. | ⬜ | ⬜ |
| Invite onboarding: accept invite + expired invite UX. | ⬜ | ⬜ |
| Profile page: name/avatar/timezone/language. | ⬜ | ⬜ |
| Change password: current password + policy hints. | ⬜ | ⬜ |
| Active sessions list + “sign out all”. | ⬜ | ⬜ |
| Login history (success/failed, timestamps/IP). | ⬜ | ⬜ |
| Admin auth middleware/guard (private routes). | ⬜ | ⬜ |
| Token storage + refresh strategy (cookie/localStorage). | ⬜ | ⬜ |
| Logout flow (очистка токена/сессии). | ⬜ | ⬜ |
| RBAC checks for admin-only GraphQL/REST. | ⬜ | ⬜ |
| RU/EN coverage for auth/profile UI + validation. | ⬜ | ⬜ |
| Localized email templates: verify/reset/invite. | ⬜ | ⬜ |
| Admin route map: `/login` `/register` `/reset` `/profile` `/security`. | ⬜ | ⬜ |
| Audit events: login/password change/session invalidation/invite. | ⬜ | ⬜ |

---

## Phase 4 — Интеграция UI‑шаблона для админок (только работы)

| Работа | Leptos | Next |
| --- | --- | --- |
| Подготовка: зафиксировать цели и scope Phase 3. | ⬜ | ⬜ |
| Инвентаризация шаблона: страницы, layout, компоненты, токены. | ⬜ | ⬜ |
| Инвентаризация текущих админок: маршруты, состояния, формы/таблицы. | ⬜ | ⬜ |
| Согласовать UI‑контракт (layout/sidebar/header, базовые компоненты). | ⬜ | ⬜ |
| Карта соответствий: страницы Template → RusToK. | ⬜ | ⬜ |
| Карта соответствий: компоненты Template → shadcn/ui/internal. | ⬜ | ⬜ |
| Карта токенов: цвета/отступы/типографика → дизайн‑токены. | ⬜ | ⬜ |
| Next.js: установить/синхронизировать зависимости шаблона. | ⬜ | ⬜ |
| Next.js: подключить layout/nav под контракт. | ⬜ | ⬜ |
| Next.js: перенести ключевые страницы (Login/Register/Reset/Profile/Security). | ⬜ | ⬜ |
| Next.js: Users list/details + Dashboard widgets. | ⬜ | ⬜ |
| Next.js: синхронизировать i18n под новые UI блоки. | ⬜ | ⬜ |
| Next.js: подключить API-клиенты и состояния (loading/error/empty). | ⬜ | ⬜ |
| Leptos: создать эквиваленты шаблонных компонентов. | ⬜ | ⬜ |
| Leptos: выровнять layout/nav с Next.js. | ⬜ | ⬜ |
| Leptos: перенести страницы тем же приоритетом. | ⬜ | ⬜ |
| Leptos: синхронизировать i18n. | ⬜ | ⬜ |
| Leptos: подключить API-слой и состояния. | ⬜ | ⬜ |
| Паритет: визуальный parity (Login/Users/Profile). | ⬜ | ⬜ |
| QA: поведение (валидация/ошибки/загрузка). | ⬜ | ⬜ |
| QA: доступность (контраст, фокус, aria). | ⬜ | ⬜ |
| QA: производительность (bundle/рендер). | ⬜ | ⬜ |
| План внедрения: порядок этапов + демонстрации. | ⬜ | ⬜ |
| План отката: фича‑флаг/релизный переключатель. | ⬜ | ⬜ |
| Definition of Done (DoD): паритет, локализация, без регрессий, документация. | ⬜ | ⬜ |

---

## Phase 2.1 — Users vertical slice (полный текст)

### Goals

This document defines **Phase 2.1** (the Users vertical slice) and is the source of truth for completion status.

Build a production-ready **Users** vertical slice with **auth + list + details + CRUD + i18n** in both admin frontends (Leptos and Next). The scope below is the minimal, iterative path to parity across both UIs.

### Scope (what we ship in this phase)

1. **i18n foundation**
   - Centralize strings to avoid growth in `locale.rs` / `messages/*.json`.
   - Agree on key namespaces: `app.*`, `auth.*`, `users.*`, `errors.*`.
   - Add translations for API errors on all pages (same approach already used in users).
2. **Users v1 (data wiring)**
   - REST auth: `/api/auth/login`, `/api/auth/me`.
   - GraphQL: `users` list (pagination), `user(id)` details.
   - Token storage (JWT) used by Leptos + Next.
3. **Admin Users table parity**
   - Columns: `email`, `name`, `role`, `status`, `created_at`.
   - Pagination, filtering, search.
4. **Users CRUD (GraphQL)**
   - Mutations for `create`, `update`, `disable` user.
   - RBAC permissions for manage/update.
5. **Shared UI/UX**
   - Layout/Navigation, Breadcrumbs, Toasts, Form patterns.

### API contracts (target shape)

#### REST

- `POST /api/auth/login`
  - Request: `{ email, password }`
  - Response: `{ access_token, refresh_token?, user }`
- `GET /api/auth/me`
  - Response: `{ user }`

#### GraphQL

```graphql
query Users($pagination: PaginationInput, $filter: UsersFilter, $search: String) {
  users(pagination: $pagination, filter: $filter, search: $search) {
    edges {
      node { id email name role status createdAt }
    }
    pageInfo { totalCount }
  }
}

query User($id: ID!) {
  user(id: $id) { id email name role status createdAt }
}

mutation CreateUser($input: CreateUserInput!) {
  createUser(input: $input) { id }
}

mutation UpdateUser($id: ID!, $input: UpdateUserInput!) {
  updateUser(id: $id, input: $input) { id }
}

mutation DisableUser($id: ID!) {
  disableUser(id: $id) { id }
}
```

### i18n conventions

**Namespaces**
- `app.*` — app-wide labels (nav, buttons).
- `auth.*` — login/forgot/2FA.
- `users.*` — list/detail/edit/new.
- `errors.*` — API error codes + fallback.

**Examples**
- `users.title`, `users.table.email`, `users.table.role`.
- `errors.auth.invalid_credentials`.

### Iteration plan (step-by-step)

#### Step 1 — i18n foundation (Leptos + Next)

**Leptos**
- Move string definitions into domain modules: `locale/app.rs`, `locale/users.rs`, `locale/errors.rs`.
- Expose a central `t(key)` helper with namespace support.
- Add translations for common API errors.

**Next**
- Split JSON files into domain subtrees (`Users`, `Auth`, `Errors`).
- Align keys with Leptos naming convention.

#### Step 2 — Auth wiring (REST)

**Leptos**
- Implement login via `POST /api/auth/login`.
- Store token (JWT) in memory + storage (TBD: localStorage/cookie).
- Use `GET /api/auth/me` for session bootstrap.

**Next**
- Mirror login behavior and token storage.
- Add middleware guard (redirect unauthenticated).

#### Step 3 — Users list + pagination

**Leptos**
- Wire GraphQL `users` query with `PaginationInput`.
- Add search and filter UI state.
- Render table with pagination.

**Next**
- Same GraphQL query + table UI.
- Server-side data load + pagination parameters.

#### Step 4 — Users detail view

**Leptos**
- Add `user(id)` query and details screen.

**Next**
- Add `user(id)` page and details view.

#### Step 5 — Users CRUD

**Backend**
- Add GraphQL mutations: `createUser`, `updateUser`, `disableUser`.
- Enforce RBAC: `users.create`, `users.update`, `users.manage`.

**Leptos + Next**
- Add create/edit forms and disable action.
- Add error handling + toast notifications.

#### Step 6 — UI/UX shared components

- Layout/Navigation parity.
- Breadcrumbs.
- Toasts.
- Form patterns.

### Deliverables checklist

- [x] i18n refactor + aligned keys
- [x] Login flow (REST) in both apps
- [x] Users list (GraphQL) with pagination + filters
- [x] User details
- [x] CRUD mutations + RBAC
- [x] Admin UI parity + shared components

### Notes / decisions

- Token storage: decide between cookie-based (server) or localStorage (client). Must align with CORS and CSRF strategy.
- Pagination: use `PaginationInput` consistently.
- Error translation: map backend error codes to `errors.*` keys with fallback.

---

## Phase 3 — Admin Auth & User Security (полный текст)

This document outlines the Phase 3 scope for **standard** multi-language admin authentication
and user security flows. It intentionally excludes SSO/OIDC/SAML to keep the first iteration
simple and production-ready.

### Goals

- Ship a production-grade login/register experience in the admin panel.
- Provide user profile management with password change and session management.
- Ensure full RU/EN localization coverage for UI, emails, and validation.
- Keep flows consistent with multi-tenant access patterns.
- Prioritize **multi-language UX** and **parallel flow support** (registration, invites,
  password reset, and profile/security actions can be developed and shipped independently).

### In Scope (MVP)

#### 1) Authentication

- **Login page**
  - Tenant slug + email + password.
  - Clear error messages for invalid credentials and missing fields.
  - Remember language choice (persisted client-side).
  - Parallelizable: can ship while registration and reset are still in progress.
- **Password reset**
  - Request reset email.
  - Reset link with token + new password.
  - Token expiration handling.
  - Parallelizable with registration and invites.
- **Email verification**
  - Verify email after registration (or optional soft-verify for internal users).
  - Resend verification email action.
  - Parallelizable with password reset.

#### 2) Registration

- **Sign-up form**
  - Email + password + tenant slug.
  - Optional name field.
  - Password strength hints.
  - Parallelizable with login, password reset, and profile.
- **Invite-based onboarding**
  - Accept invitation links with role pre-selected.
  - Expired invitation handling.
  - Can be delivered separately from open registration.

#### 3) User Profile & Security

- **Profile page**
  - Update name, avatar, timezone, preferred language.
  - Separate user-facing language preference from admin default language.
- **Change password**
  - Requires current password.
  - Show password policy hints.
  - Parallelizable with session list.
- **Active sessions**
  - List recent sessions (device, IP, last active).
  - “Sign out all” action.
  - Parallelizable with login history.
- **Login history**
  - Successful/failed logins with timestamps and IPs.
  - Use localized date/time formatting.

### Parallel Delivery Tracks

Each track is self-contained, can be shipped independently, and shares a small set
of reusable UI components (inputs, validation, callouts, empty states).

#### Track A — Auth Core

- Login page (tenant + email + password).
- Auth errors and loading states.
- Language switch + persistence.

#### Track B — Password Recovery

- Request reset email.
- Reset form with token + new password.
- Token expiry UI.

#### Track C — Registration & Invites

- Sign-up form (with tenant).
- Invite acceptance flow.
- Email verification + resend action.

#### Track D — Profile & Security

- Profile settings (name, avatar, timezone, language).
- Change password flow.
- Sessions + login history.

### Localization (RU/EN)

- All auth/profile UI strings are localized.
- Email templates are localized: verify email, reset password, invite.
- Locale selection persists across sessions.
- Ensure validation errors are localized and context-aware (field + error type).

### Data & Audit

- Track audit events for:
  - Logins (success/failure).
  - Password changes.
  - Session invalidations.
  - Email verification changes.
  - Invite accepted/expired events.

### UX Notes

- Keep forms minimal and mobile-friendly.
- Use inline validation with precise messages.
- Use clear empty states for sessions/log history.
- Prefer UX patterns that allow teams to develop features in parallel:
  - shared auth UI components,
  - isolated endpoints per flow,
  - independent feature flags.

### Deliverables Checklist

- RU/EN dictionary coverage for all auth/profile UI + validation.
- Email templates in RU/EN for: verify, reset, invite.
- Admin route map: `/login`, `/register`, `/reset`, `/profile`, `/security`.
- Minimal audit log schema for auth events.

### Out of Scope (Phase 3)

- SSO (OIDC/SAML).
- Passwordless magic links.
- 2FA / TOTP (planned for future phase).

---

## Phase 3 — Admin architecture & contracts (полный текст)

This document describes the **implemented architecture** for Phase 3 admin auth/security
flows and the contracts that both admin frontends should follow.

### Runtime layers

#### Backend (`apps/server`)

Phase 3 logic lives in `apps/server/src/controllers/auth.rs` and `apps/server/src/auth.rs`.

Core responsibilities:
- Credentials login/register and token issuance.
- Password reset token generation/validation.
- Profile update.
- Password change.
- Session listing/history/revoke-all.

#### Leptos admin (`apps/admin`)

Leptos app calls server REST endpoints through shared helpers in `apps/admin/src/api/mod.rs`.
Auth state is kept in `AuthContext` with persisted:
- `rustok-admin-token`
- `rustok-admin-user`
- `rustok-admin-tenant`

This keeps all protected calls tenant-scoped without requiring repeated tenant input.

#### Next admin (`apps/next-admin`)

Next app uses locale routes (`/[locale]/*`) and cookie-based auth context:
- `rustok-admin-token`
- `rustok-admin-tenant`

Client views use direct fetch calls to `/api/auth/*`. A shared helper
`src/lib/client-auth.ts` centralizes cookie parsing.

### HTTP contract map (Phase 3)

All endpoints are under `/api/auth`.

**Public endpoints**
- `POST /login`
- `POST /register`
- `POST /reset/request`
- `POST /reset/confirm`

**Protected endpoints (Bearer + tenant header)**
- `GET /me`
- `POST /profile`
- `POST /change-password`
- `GET /sessions`
- `GET /history`
- `POST /sessions/revoke-all`

Tenant scoping is via `X-Tenant-Slug` + token tenant validation.

### Security model

#### Access token claims

Access token includes user, tenant, role, and `session_id`.
`session_id` is propagated into `CurrentUser` extractor so protected handlers
can preserve current session when revoking others.

#### Password reset token claims

Reset flow uses a dedicated JWT claim model (`PasswordResetClaims`) with:
- `tenant_id`
- `sub` (email)
- `purpose=password_reset`
- expiration (`DEFAULT_RESET_TOKEN_TTL_SECS`)

Confirm endpoint validates signature, expiry, purpose, and tenant match.

#### Session lifecycle

- Login/Register create refresh session records.
- `change-password` revokes all sessions except current.
- `sessions/revoke-all` revokes all sessions except current.
- `sessions` returns active sessions.
- `history` returns recent session activity entries.

### Frontend behavior contract

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

### Current implementation status

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

### Sequence snapshots

#### Register

1. UI sends `POST /api/auth/register` + `X-Tenant-Slug`.
2. Server creates user and initial session.
3. Server returns access token + user info.
4. Frontend persists token + tenant + user.

#### Reset

1. UI sends `POST /api/auth/reset/request`.
2. Server returns generic success (and optional token in demo mode).
3. UI sends `POST /api/auth/reset/confirm` with token + new password.
4. Server validates reset token claims and updates password hash.

#### Security revoke-all

1. UI sends `POST /api/auth/sessions/revoke-all` with Bearer token.
2. Server resolves `CurrentUser` with current `session_id`.
3. Server revokes all other sessions for user/tenant.
4. UI refreshes `GET /api/auth/sessions`.

---

## Phase 3 — Gap analysis & parity plan (полный текст)

This section compares the current implementation with the target Phase 3 scope
and adds parity guidance for a unified admin UX.

### Scope source

Phase 3 target scope is defined in this document:

- Auth core (`/login`)
- Password recovery (`/reset`)
- Registration and invites (`/register` + invite acceptance)
- Profile and security (`/profile`, `/security`)
- RU/EN localization for UI, validation, and auth emails
- Audit events for auth/security actions

### Status legend

- ✅ Done — feature works end-to-end in the app
- 🟡 Partial — route/UI exists, but endpoint wiring or behavior is incomplete
- ❌ Missing — feature not yet implemented

### Route map parity snapshot

| Route | Leptos admin (`apps/admin`) | Next admin (`apps/next-admin`) | Notes |
| --- | --- | --- | --- |
| `/login` | ✅ | ✅ (`/[locale]/login`) | Both implement tenant + email + password login flow. |
| `/register` | ✅ | ✅ (`/[locale]/register`) | API-wired in both admin apps. |
| `/reset` | ✅ | ✅ (`/[locale]/reset`) | Reset request/confirm wired in both admin apps. |
| `/profile` | ✅ | ✅ (`/[locale]/profile`) | Profile update endpoint wired in both admin apps. |
| `/security` | ✅ | ✅ (`/[locale]/security`) | Sessions/history/change-password/revoke-all are API-wired. |

### Detailed phase checklist

#### Track A — Auth core

| Capability | Leptos | Next | Gap / action |
| --- | --- | --- | --- |
| Login form fields (tenant, email, password) | ✅ | ✅ | Keep validation and error mapping identical. |
| Login request to backend | ✅ | ✅ | Both already call `/api/auth/login`. |
| Guard unauthenticated routes | ✅ | ✅ | Keep redirect behavior aligned for all protected routes. |
| Locale switch + persistence | ✅ | 🟡 | Next has locale routes, but explicit auth-locale persistence policy should match Leptos. |

#### Track B — Password recovery

| Capability | Leptos | Next | Gap / action |
| --- | --- | --- | --- |
| Reset request UI | ✅ | ✅ | Implemented in both apps with tenant-aware request. |
| Reset token + new password flow | ✅ | ✅ | Both use `/api/auth/reset/confirm`. |
| Token expiry UX | 🟡 | 🟡 | Contract supports expiry; dedicated UX state can be improved. |

#### Track C — Registration & invites

| Capability | Leptos | Next | Gap / action |
| --- | --- | --- | --- |
| Registration form | ✅ | ✅ | Both use `/api/auth/register`. |
| Invite acceptance | ❌ | ❌ | Add invite endpoint + page in both apps. |
| Email verification + resend | ❌ | ❌ | Add verify/resend flow and localized feedback. |

#### Track D — Profile & security

| Capability | Leptos | Next | Gap / action |
| --- | --- | --- | --- |
| Profile editing (name, avatar, timezone, language) | 🟡 | 🟡 | Name update is wired; avatar/timezone/language persistence still pending backend fields. |
| Change password | ✅ | ✅ | Both call `/api/auth/change-password`. |
| Active sessions list | ✅ | ✅ | Both call `/api/auth/sessions`. |
| Login history | ✅ | ✅ | Both call `/api/auth/history`; pagination/audit enrichment remains future work. |
| Sign out all sessions | ✅ | ✅ | Both call `/api/auth/sessions/revoke-all`. |

### Shared UX and component-system parity (shadcn/ui)

To keep identical look-and-feel and behavior in both admin apps, enforce a shared
component contract independent of framework:

1. **Design token parity**
   - Color scale, spacing, radius, typography, shadows
   - Semantic tokens for states: `success`, `warning`, `destructive`, `muted`
2. **Component behavior parity**
   - Input validation timing (`onBlur` / `onSubmit`)
   - Button loading/disabled semantics
   - Alert and inline error presentation
3. **Form contract parity**
   - Same field order and labels for auth/security forms
   - Same backend error-code mapping to i18n keys (`errors.*`)
4. **State UX parity**
   - Loading skeletons/spinners
   - Empty states (sessions/history)
   - Retry/error banners
5. **Accessibility parity**
   - Focus ring + keyboard navigation
   - Label/input associations
   - `aria-live` for async validation and submit errors

### Practical implementation recommendation

Create a shared "Admin UI Contract" doc with:

- canonical component names (`Button`, `Input`, `Select`, `Alert`, `Card`, `Dialog`)
- required props/states and interaction rules
- visual snapshots for default/hover/focus/error/disabled states
- page-level wireframes for `login/register/reset/profile/security`

Then align:

- `apps/next-admin` components (already shadcn-based)
- `apps/admin` components to the same contract (shadcn-style API and states)

### Priority delivery plan (2 sprints)

> Note: route rollout and core endpoint wiring from the earlier plan are complete
> (`/register`, `/reset`, `/profile`, `/security` in both admin apps). This plan
> is intentionally trimmed to the remaining Phase 3 scope.

#### Sprint 1 (scope-closure: invites + verification)

1. Implement invite acceptance end-to-end (backend contract + both admin UIs).
2. Add invite expiry handling and localized user-facing states.
3. Implement email verification flow + resend action (backend + both UIs).
4. Localize verification/invite/reset transactional templates in RU/EN.

#### Sprint 2 (security observability + profile completeness)

1. Emit/store auth-security audit events (login success/fail, password changed,
   session invalidation, verification changes, invite accepted/expired).
2. Surface a dedicated audit feed in admin UI (separate from session history).
3. Complete profile model persistence: avatar, timezone, preferred language,
   with explicit user-facing vs admin UI language behavior.
4. Add explicit reset-token-expired UX state with recovery CTA in both apps.

### Definition of done (Phase 3 admin)

Phase 3 can be considered done when:

- Both admin apps expose and protect all target routes:
  `/login`, `/register`, `/reset`, `/profile`, `/security`.
- All auth/security flows are API-wired (not static/demo-only).
- RU/EN coverage is complete for UI + validation + transactional auth emails.
- Audit events are emitted and visible for login/security actions.
- UI parity checks confirm equivalent states and interactions in Leptos and Next.

---

## Phase 4 — Интеграция UI‑шаблона для админок (полный текст)

Цель: аккуратно «натянуть» новый UI-шаблон на обе админки (Leptos и Next.js), сохранив единый UX, совместимость с текущими API и требованиями Phase 3.

### 1. Подготовка и аудит

1. Зафиксировать исходные цели (scope Phase 3 и требования к паритету).
   - Документы: см. разделы Phase 3 в этом документе (scope/architecture/gap analysis).
2. Снять инвентаризацию шаблона:
   - Список страниц, layouts, UI-компонентов, токены дизайна, типографика.
   - Зависимости (Tailwind/shadcn/ui/иконки/таблицы/формы).
3. Снять инвентаризацию текущих админок:
   - Реальные маршруты (`/login`, `/register`, `/reset`, `/profile`, `/security`, `/users`).
   - Основные состояния (loading/empty/error), формы и таблицы.
4. Согласовать «UI контракт» между админками:
   - Структура страниц и базовые шаблоны (layout/sidebar/header).
   - Компоненты: кнопки, инпуты, таблицы, модалки, алерты.

### 2. Карта соответствий (Template → RusToK)

1. Таблица соответствий страниц:
   - Шаблонные экраны → реальные роуты админки.
2. Таблица соответствий компонентов:
   - Элементы шаблона → shadcn/ui или внутренние компоненты.
3. Карта токенов:
   - Цвета, отступы, типографика → дизайн-токены, используемые в обоих проектах.

### 3. Интеграция в Next.js админку (`apps/next-admin`)

1. Установить/синхронизировать зависимости шаблона.
2. Подключить layout и навигацию:
   - Привести sidebar и topbar к контракту и роутингу RusToK.
3. Перенести ключевые страницы в порядке приоритета:
   1) Login/Register/Reset/Profile/Security
   2) Users list/details
   3) Dashboard widgets
4. Синхронизировать i18n:
   - Интеграция `next-intl` сообщений под новые UI блоки.
5. Подключить API-клиенты и состояния:
   - Загрузка, ошибки, пустые состояния, пагинация.

### 4. Интеграция в Leptos админку (`apps/admin`)

1. Создать Leptos-эквиваленты шаблонных компонентов:
   - shadcn-style API и состояния (loading/empty/error/disabled).
2. Выровнять layout и навигацию:
   - Контракт совпадает с Next.js версией.
3. Перенести страницы тем же приоритетом, что и для Next.js.
4. Синхронизировать i18n:
   - Добавить/обновить строки локализации для новых UI блоков.
5. Подключить API-слой и контракты состояния.

### 5. Паритет и контроль качества

1. Проверить визуальный паритет (pixel/spacing):
   - Сравнить ключевые экраны (Login, Users, Profile).
2. Проверить поведение:
   - Валидация форм, обработка ошибок, загрузка данных.
3. Проверить доступность:
   - Контраст, фокус, aria-атрибуты.
4. Проверить производительность:
   - Размер бандла, скорость рендера ключевых экранов.

### 6. План внедрения и отката

1. Внедрять по шагам: сначала Next.js, затем Leptos (или параллельно по экранам).
2. Для каждого шага:
   - Демонстрационный прогон (dev) + визуальная проверка.
3. Откат:
   - Фича-флаг или переключение через ветки/релизы.

### 7. Definition of Done (DoD)

- Все Phase 3 маршруты визуально соответствуют шаблону в обеих админках.
- Компоненты имеют единый UI контракт и состояния.
- Локализация покрывает новые UI строки.
- Нет регрессий по API/валидации/роутингу.
- Документация обновлена и содержит ссылки на шаблон и план интеграции.
