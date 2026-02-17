# Leptos Admin Panel — Review & Improvement Plan

## Context

Leptos admin (`apps/admin`) is the primary admin panel for RusTok. The Next.js admin (`apps/next-admin`) serves as reference. After recent commits: `_new.rs` duplicates removed, auth migrated to `leptos-auth` crate, Clerk replaced with NextAuth+JWT in Next.js, Users page added to Next.js admin. However, critical issues remain: tokens expire silently, dashboard stats are hardcoded, filtering is duplicated client/server, no RBAC, no CRUD for users.

The project has a rich crate infrastructure (`leptos-ui`, `leptos-forms`, `leptos-graphql`, `leptos-table`, `leptos-use`, `leptos-chartistry`, `leptos-shadcn-pagination`) that is **almost entirely unused** in the admin app.

---

## Phase 1: Auth Hardening + Dashboard Data (Critical)

### 1.1 Store refresh_token and expires_in in AuthSession

**Files:**
- `crates/leptos-auth/src/lib.rs` — add fields `refresh_token: String`, `expires_at: i64` to `AuthSession`
- `crates/leptos-auth/src/api.rs:138-151` — remove `#[allow(dead_code)]` from `refresh_token`, `token_type`, `expires_in` in `AuthPayload`; in `sign_in` (~line 236) and `sign_up` (~line 280) populate new fields: `expires_at = now() + expires_in`
- Serialization via serde + gloo_storage will automatically pick up new fields

### 1.2 Token expiration checking and auto-refresh

**Files:**
- `crates/leptos-auth/src/context.rs` — add `is_token_expired()` method to `AuthContext` (check `now >= expires_at - 60`)
- In `AuthProvider` add interval via `leptos_use::use_interval_fn` (60sec): if `remaining < 300` — refresh, if `<= 0` — sign_out
- `crates/leptos-auth/src/api.rs` — in `refresh_token()` use `session.refresh_token` instead of `session.token`
- `crates/leptos-auth/src/hooks.rs` — add `use_is_token_valid() -> Signal<bool>`

### 1.3 Dashboard — real data from GraphQL

**File:** `apps/admin/src/pages/dashboard.rs`
- Replace hardcoded stats (lines 13-56: "28", "12", "128ms", "7") with `Resource::new` + GraphQL `DashboardStats` query
- Add `<Suspense>` with skeleton-fallback for cards
- Activity feed — mark as placeholder or connect to `recentActivity` query if endpoint exists

### 1.4 Centralize GraphQL queries

**New file:** `apps/admin/src/api/queries.rs`
- Extract all query strings and persisted hashes from `users.rs:137,149` and `user_details.rs:65` into one module
- Remove magic hash strings from components

### 1.5 Centralize API URL

**Files:**
- `apps/admin/src/api/mod.rs:10-24` — add `option_env!("RUSTOK_GRAPHQL_URL")` as compile-time override
- `crates/leptos-auth/src/api.rs:180-196` — duplicates `get_graphql_url()` → import from shared location

---

## Phase 2: Users Page — Server-Side Filtering + Debounce (High)

### 2.1 Fix Resource dependencies

**File:** `apps/admin/src/pages/users.rs:125-126`
- Resource depends only on `(refresh_counter, page, limit)` — **does not react** to `search_query`, `role_filter`, `status_filter`
- Add `debounced_search`, `role_filter`, `status_filter` to dependency tuple

### 2.2 Remove duplicate client-side filtering

**File:** `apps/admin/src/pages/users.rs:250-273`
- Server already receives filter/search variables → client-side `.filter()` is redundant
- Replace with direct `.map()` over `edges`

### 2.3 Debounce on search

**File:** `apps/admin/src/pages/users.rs`
- Use `leptos_use::use_debounce_fn` (already in deps) with 300ms delay for search input
- Role/status filters don't need debounce (discrete actions)

### 2.4 Reset page on filter change

- On change of `debounced_search`, `role_filter`, `status_filter` → `set_page(1)`

### 2.5 Use `leptos_ui::Badge` for status column

**File:** `apps/admin/src/pages/users.rs:298`
- Replace inline `<span>` with `Badge` using `BadgeVariant::Success`/`BadgeVariant::Secondary`

---

## Phase 3: User Details CRUD + Form Validation (High)

### 3.1 Edit mode for user details

**File:** `apps/admin/src/pages/user_details.rs`
- Add signals `is_editing`, `edit_name`, `edit_role`, `edit_status`
- "Edit" button next to "Back to Users"
- In edit mode: show `Input` instead of text, "Save"/"Cancel" buttons
- `UPDATE_USER_MUTATION` via GraphQL
- Validation via `leptos_forms::Validator`

### 3.2 Delete with confirmation

**File:** `apps/admin/src/pages/user_details.rs`
- `DELETE_USER_MUTATION` + `show_delete_confirm` signal
- Inline confirmation overlay (backdrop + confirm/cancel)
- After deletion — navigate to `/users`

### 3.3 Password strength validation in registration

**File:** `apps/admin/src/pages/register.rs`
- Use `leptos_forms::Validator::new().required().min_length(8).custom(...)` — uppercase + lowercase + digit
- Email validation via `.email()`
- Show errors below fields

### 3.4 Improve login error handling

**File:** `apps/admin/src/pages/login.rs`
- Map `AuthError` variants to translated messages via `translate()`

---

## Phase 4: Layout, Navigation, UI Polish (Medium)

### 4.1 Collapsible sidebar + mobile drawer

**File:** `apps/admin/src/components/layout/sidebar.rs`
- `SidebarState` context: `collapsed: RwSignal<bool>`, `mobile_open: RwSignal<bool>`
- `leptos_use::use_media_query("(max-width: 768px)")` for auto-collapse
- Collapsed: `w-16` icons only; expanded: `w-64` with labels
- Mobile: overlay drawer with backdrop

**File:** `apps/admin/src/components/layout/app_layout.rs`
- Wrap in `SidebarProvider`, `ml-16`/`ml-64` based on state

### 4.2 Dynamic breadcrumbs

**File:** `apps/admin/src/components/layout/header.rs:12-16`
- Replace static "RusTok / Admin" with `use_breadcrumbs()` from `use_location()`
- Path mapping: `/users` → Dashboard > Users, `/users/:id` → Dashboard > Users > Detail

### 4.3 SVG icons instead of emoji in user menu

**File:** `apps/admin/src/components/features/auth/user_menu.rs:93,96,106`
- Replace "👤", "🔒", "🚪" with SVG (pattern from sidebar `NavIcon`)
- Translate hardcoded strings "Profile", "Security", "Sign Out" via `translate()`

### 4.4 RBAC navigation filtering

**File:** `apps/admin/src/components/layout/sidebar.rs`
- Add `required_role: Option<&str>` to nav items
- Filter by `current_user.role` (field already exists in `AuthUser` but never checked)

### 4.5 i18n for sidebar labels

- "Overview", "Management", "Account" → `translate("app.nav.group.*")`
- Add keys to `locales/en.json` and `locales/ru.json`

---

## Phase 5: Profile, Security, UI Components (Medium)

### 5.1 Fix profile defaults

**File:** `apps/admin/src/pages/profile.rs`
- Timezone (line 67): `"Europe/Moscow"` → detect via `js_sys::Intl::DateTimeFormat`
- Locale (line 68): `"ru"` → from current `use_locale()`
- Add avatar preview: `<img>` with `on:error` fallback

### 5.2 Session management on Security page

**File:** `apps/admin/src/pages/security.rs:186-188`
- Replace placeholder "coming soon" with GraphQL query `mySessions`
- Session list: IP, userAgent, lastActiveAt, isCurrent badge, "Revoke" button

### 5.3 Skeleton component in leptos-ui

**New file:** `crates/leptos-ui/src/skeleton.rs`
- `<Skeleton class="h-32" />` — `animate-pulse rounded-md bg-slate-200`
- Export from `crates/leptos-ui/src/lib.rs`
- Use as fallback in dashboard, users, user_details

### 5.4 Select and Toast components

**New:** `crates/leptos-ui/src/select.rs` — basic Select dropdown for profile (timezone, locale)
**New:** `apps/admin/src/components/ui/toast.rs` — ToastContext with `success()`/`error()`, auto-dismiss 5sec, fixed-position stack

---

## Phase 6: Error Boundaries + Role Guards (Medium)

### 6.1 Error boundaries around pages

**File:** `apps/admin/src/app.rs`
- Wrap each Route view in `<ErrorBoundary fallback=...>`
- Or create `PageErrorBoundary` wrapper

### 6.2 Loading skeletons for all pages

- Replace spinners in `<Suspense fallback>` with skeleton layouts
- Users: skeleton-table, Dashboard: skeleton-cards, User detail: skeleton-card

### 6.3 Role-based route guards

**File:** `crates/leptos-auth/src/components.rs`
- New component `RequireRole { role: String }` — checks `current_user.role`
- Use in `app.rs` for `/users` route: `<RequireRole role="ADMIN">`

---

## Phase 7: Charts, Dark Mode (Low)

### 7.1 Dashboard charts
- `leptos-chartistry` (in deps) — area chart + bar chart

### 7.2 Dark mode
- `ThemeContext` (light/dark/system), `leptos_use::use_preferred_dark`, toggle `dark` class on `<html>`

### 7.3 Command palette (Cmd+K)
- Global keyboard listener, search through nav items

---

## Crate Utilization Matrix

| Crate | Current | After Plan |
|-------|---------|------------|
| `leptos-ui` (Button, Input, Card, Badge, Label, Separator) | Admin uses its own custom Button/Input | + Badge in users, + Skeleton (new), + Select (new) |
| `leptos-forms` (Validator, FormContext) | **Unused** | Validation in register, profile, user edit |
| `leptos-graphql` (use_query, use_mutation) | Only raw `execute` | Consider hooks for new queries |
| `leptos-table` (TableState) | **Unused** | State management for users table |
| `leptos-shadcn-pagination` | **Unused** | Users page pagination |
| `leptos-use` | **Unused** | `use_debounce_fn`, `use_interval_fn`, `use_media_query` |
| `leptos-chartistry` | **Unused** | Dashboard charts (Phase 7) |
| `leptos-zod` | **Unused** | Map API errors → form fields |

---

## Execution Order

```
Phase 1 (Auth + Dashboard) ← foundation, do first
   ↓
Phase 2 (Users filtering) ← depends on token fix
   ↓
Phase 3 (User CRUD + Forms) ← depends on clean data flow
   ↓ (parallel)
Phase 4 (Layout + Nav) ← independent of 2-3
Phase 6 (Error boundaries) ← independent
   ↓
Phase 5 (Profile + Security + UI) ← depends on Toast/Select from Phase 4-5
   ↓
Phase 7 (Charts + Dark mode) ← lowest priority
```

## Verification

1. `cargo build -p admin` — compile after each phase
2. Test auth flow: login → token refresh → auto-logout on expired
3. Dashboard: verify stats come from API (or graceful fallback)
4. Users: search with debounce, filters re-fetch data, pagination resets
5. User details: edit → save → verify, delete → confirm → redirect
6. Responsive: sidebar collapse on mobile, breadcrumbs correct on all pages
7. i18n: switch EN↔RU, verify all new strings
