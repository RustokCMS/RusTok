# Admin Phase 3 gap analysis (Leptos + Next.js)

This document compares the current implementation with the target scope from
`docs/UI/admin-auth-phase3.md` and adds parity guidance for a unified admin UX.

## Контекст админок

- **Описание:** сводка расхождений между админками и контрольная карта закрытия Phase 3.
- **Стек:** Leptos CSR (`apps/admin`) + Next.js App Router (`apps/next-admin`), TailwindCSS, shadcn/ui, leptos-shadcn-ui, общие дизайн‑токены.
- **Ссылки:** [UI документы](./) • [UI parity](./ui-parity.md) • [IU библиотеки](../../IU/README.md)

Implementation architecture is documented in `docs/UI/admin-phase3-architecture.md`.

> Context: the project is converging on a shared admin look/behavior and a unified
> component approach (`shadcn/ui`-style design system in both admin apps).

## Scope source

Phase 3 target scope is defined in `docs/UI/admin-auth-phase3.md`:

- Auth core (`/login`)
- Password recovery (`/reset`)
- Registration and invites (`/register` + invite acceptance)
- Profile and security (`/profile`, `/security`)
- RU/EN localization for UI, validation, and auth emails
- Audit events for auth/security actions

## Status legend

- ✅ Done — feature works end-to-end in the app
- 🟡 Partial — route/UI exists, but endpoint wiring or behavior is incomplete
- ❌ Missing — feature not yet implemented

## Route map parity snapshot

| Route | Leptos admin (`apps/admin`) | Next admin (`apps/next-admin`) | Notes |
| --- | --- | --- | --- |
| `/login` | ✅ | ✅ (`/[locale]/login`) | Both implement tenant + email + password login flow. |
| `/register` | ✅ | ✅ (`/[locale]/register`) | API-wired in both admin apps. |
| `/reset` | ✅ | ✅ (`/[locale]/reset`) | Reset request/confirm wired in both admin apps. |
| `/profile` | ✅ | ✅ (`/[locale]/profile`) | Profile update endpoint wired in both admin apps. |
| `/security` | ✅ | ✅ (`/[locale]/security`) | Sessions/history/change-password/revoke-all are API-wired. |

## Detailed phase checklist

### Track A — Auth core

| Capability | Leptos | Next | Gap / action |
| --- | --- | --- | --- |
| Login form fields (tenant, email, password) | ✅ | ✅ | Keep validation and error mapping identical. |
| Login request to backend | ✅ | ✅ | Both already call `/api/auth/login`. |
| Guard unauthenticated routes | ✅ | ✅ | Keep redirect behavior aligned for all protected routes. |
| Locale switch + persistence | ✅ | 🟡 | Next has locale routes, but explicit auth-locale persistence policy should match Leptos. |

### Track B — Password recovery

| Capability | Leptos | Next | Gap / action |
| --- | --- | --- | --- |
| Reset request UI | ✅ | ✅ | Implemented in both apps with tenant-aware request. |
| Reset token + new password flow | ✅ | ✅ | Both use `/api/auth/reset/confirm`. |
| Token expiry UX | 🟡 | 🟡 | Contract supports expiry; dedicated UX state can be improved. |

### Track C — Registration & invites

| Capability | Leptos | Next | Gap / action |
| --- | --- | --- | --- |
| Registration form | ✅ | ✅ | Both use `/api/auth/register`. |
| Invite acceptance | ❌ | ❌ | Add invite endpoint + page in both apps. |
| Email verification + resend | ❌ | ❌ | Add verify/resend flow and localized feedback. |

### Track D — Profile & security

| Capability | Leptos | Next | Gap / action |
| --- | --- | --- | --- |
| Profile editing (name, avatar, timezone, language) | 🟡 | 🟡 | Name update is wired; avatar/timezone/language persistence still pending backend fields. |
| Change password | ✅ | ✅ | Both call `/api/auth/change-password`. |
| Active sessions list | ✅ | ✅ | Both call `/api/auth/sessions`. |
| Login history | ✅ | ✅ | Both call `/api/auth/history`; pagination/audit enrichment remains future work. |
| Sign out all sessions | ✅ | ✅ | Both call `/api/auth/sessions/revoke-all`. |

## Shared UX and component-system parity (shadcn/ui)

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

## Priority delivery plan (2 sprints)

> Note: route rollout and core endpoint wiring from the earlier plan are complete
> (`/register`, `/reset`, `/profile`, `/security` in both admin apps). This plan
> is intentionally trimmed to the remaining Phase 3 scope.

### Sprint 1 (scope-closure: invites + verification)

1. Implement invite acceptance end-to-end (backend contract + both admin UIs).
2. Add invite expiry handling and localized user-facing states.
3. Implement email verification flow + resend action (backend + both UIs).
4. Localize verification/invite/reset transactional templates in RU/EN.

### Sprint 2 (security observability + profile completeness)

1. Emit/store auth-security audit events (login success/fail, password changed,
   session invalidation, verification changes, invite accepted/expired).
2. Surface a dedicated audit feed in admin UI (separate from session history).
3. Complete profile model persistence: avatar, timezone, preferred language,
   with explicit user-facing vs admin UI language behavior.
4. Add explicit reset-token-expired UX state with recovery CTA in both apps.

## Definition of done (Phase 3 admin)

Phase 3 can be considered done when:

- Both admin apps expose and protect all target routes:
  `/login`, `/register`, `/reset`, `/profile`, `/security`.
- All auth/security flows are API-wired (not static/demo-only).
- RU/EN coverage is complete for UI + validation + transactional auth emails.
- Audit events are emitted and visible for login/security actions.
- UI parity checks confirm equivalent states and interactions in Leptos and Next.
