# Phase 2.1 — Users vertical slice (Leptos + Next)

## Goals

This document defines **Phase 2.1** (the Users vertical slice) and is the source of truth for completion status.

Build a production-ready **Users** vertical slice with **auth + list + details + CRUD + i18n** in both admin frontends (Leptos and Next). The scope below is the minimal, iterative path to parity across both UIs.

## Scope (what we ship in this phase)

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

## API contracts (target shape)

### REST

- `POST /api/auth/login`
  - Request: `{ email, password }`
  - Response: `{ access_token, refresh_token?, user }`
- `GET /api/auth/me`
  - Response: `{ user }`

### GraphQL

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

## i18n conventions

**Namespaces**
- `app.*` — app-wide labels (nav, buttons).
- `auth.*` — login/forgot/2FA.
- `users.*` — list/detail/edit/new.
- `errors.*` — API error codes + fallback.

**Examples**
- `users.title`, `users.table.email`, `users.table.role`.
- `errors.auth.invalid_credentials`.

## Iteration plan (step-by-step)

### Step 1 — i18n foundation (Leptos + Next)

**Leptos**
- Move string definitions into domain modules: `locale/app.rs`, `locale/users.rs`, `locale/errors.rs`.
- Expose a central `t(key)` helper with namespace support.
- Add translations for common API errors.

**Next**
- Split JSON files into domain subtrees (`Users`, `Auth`, `Errors`).
- Align keys with Leptos naming convention.

### Step 2 — Auth wiring (REST)

**Leptos**
- Implement login via `POST /api/auth/login`.
- Store token (JWT) in memory + storage (TBD: localStorage/cookie).
- Use `GET /api/auth/me` for session bootstrap.

**Next**
- Mirror login behavior and token storage.
- Add middleware guard (redirect unauthenticated).

### Step 3 — Users list + pagination

**Leptos**
- Wire GraphQL `users` query with `PaginationInput`.
- Add search and filter UI state.
- Render table with pagination.

**Next**
- Same GraphQL query + table UI.
- Server-side data load + pagination parameters.

### Step 4 — Users detail view

**Leptos**
- Add `user(id)` query and details screen.

**Next**
- Add `user(id)` page and details view.

### Step 5 — Users CRUD

**Backend**
- Add GraphQL mutations: `createUser`, `updateUser`, `disableUser`.
- Enforce RBAC: `users.create`, `users.update`, `users.manage`.

**Leptos + Next**
- Add create/edit forms and disable action.
- Add error handling + toast notifications.

### Step 6 — UI/UX shared components

- Layout/Navigation parity.
- Breadcrumbs.
- Toasts.
- Form patterns.

## Deliverables checklist

- [ ] i18n refactor + aligned keys
- [ ] Login flow (REST) in both apps
- [ ] Users list (GraphQL) with pagination + filters
- [x] User details
- [ ] CRUD mutations + RBAC
- [ ] Admin UI parity + shared components

## Notes / decisions

- Token storage: decide between cookie-based (server) or localStorage (client). Must align with CORS and CSRF strategy.
- Pagination: use `PaginationInput` consistently.
- Error translation: map backend error codes to `errors.*` keys with fallback.
