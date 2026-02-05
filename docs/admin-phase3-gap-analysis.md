# Admin Phase 3 gap analysis (Leptos + Next.js)

This document compares the current implementation with the target scope from
`docs/admin-auth-phase3.md` and adds parity guidance for a unified admin UX.

> Context: the project is converging on a shared admin look/behavior and a unified
> component approach (`shadcn/ui`-style design system in both admin apps).

## Scope source

Phase 3 target scope is defined in `docs/admin-auth-phase3.md`:

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
| `/register` | ✅ | ❌ | Next app has no register route yet. |
| `/reset` | ✅ | ❌ | Next app has no reset route yet. |
| `/profile` | ✅ | ❌ | Next app has no profile route yet. |
| `/security` | ✅ | ❌ | Next app has no security route yet. |

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
| Reset request UI | ✅ (UI) | ❌ | Implement `/{locale}/reset` in Next with same states. |
| Reset token + new password flow | 🟡 | ❌ | Confirm backend contract and wire both apps to same endpoints. |
| Token expiry UX | 🟡 | ❌ | Add explicit expired-token state and recovery CTA. |

### Track C — Registration & invites

| Capability | Leptos | Next | Gap / action |
| --- | --- | --- | --- |
| Registration form | ✅ (UI) | ❌ | Implement `/{locale}/register` in Next. |
| Invite acceptance | ❌ | ❌ | Add invite endpoint + page in both apps. |
| Email verification + resend | ❌ | ❌ | Add verify/resend flow and localized feedback. |

### Track D — Profile & security

| Capability | Leptos | Next | Gap / action |
| --- | --- | --- | --- |
| Profile editing (name, avatar, timezone, language) | 🟡 | ❌ | Leptos has profile page shell; complete backend wiring and mirror in Next. |
| Change password | 🟡 | ❌ | Add backend mutation/endpoint integration and policy errors. |
| Active sessions list | 🟡 | ❌ | Replace demo/static records with API data. |
| Login history | 🟡 | ❌ | Add paginated audit feed + localized timestamps. |
| Sign out all sessions | 🟡 | ❌ | Wire action to server session invalidation endpoint. |

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

### Sprint 1 (highest risk reduction)

1. Add Next routes: `/{locale}/register`, `/{locale}/reset`, `/{locale}/profile`, `/{locale}/security`.
2. Unify error mapping matrix (`401`, validation, network, unknown) across both apps.
3. Finalize component contract for auth pages (Button/Input/Alert/Card).
4. Wire reset and register endpoints end-to-end (including locale-aware messages).

### Sprint 2 (security completion)

1. Sessions and login history API integration for both apps.
2. Sign-out-all + password change flows with audit events.
3. Invite acceptance and email verification/resend.
4. Visual regression checks to enforce parity on phase-3 routes.

## Definition of done (Phase 3 admin)

Phase 3 can be considered done when:

- Both admin apps expose and protect all target routes:
  `/login`, `/register`, `/reset`, `/profile`, `/security`.
- All auth/security flows are API-wired (not static/demo-only).
- RU/EN coverage is complete for UI + validation + transactional auth emails.
- Audit events are emitted and visible for login/security actions.
- UI parity checks confirm equivalent states and interactions in Leptos and Next.
