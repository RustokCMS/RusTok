# UI parity plan (Leptos + Next.js)

This document describes how we keep **identical UI** across the Leptos and Next.js apps (admin + storefront)
using **shadcn/ui** and **leptos-shadcn-ui** with shared design tokens and a parallel delivery plan.

## Goals

- Ship **pixel-consistent UI** between Leptos and Next.js for admin and storefront experiences.
- Use a **single component philosophy** (shadcn-style, headless + tokens) to avoid drift.
- Enable **parallel development** with clear parity checks and documentation.
- Accept small deviations when a Leptos UI library lacks a 1:1 feature, but document every gap.

## Scope

Applies to the following apps:

- Admin: `apps/admin` (Leptos CSR) + `apps/next-admin` (Next.js).
- Storefront: `apps/storefront` (Leptos SSR) + `apps/next-frontend` (Next.js).

## Library decision

- **Next.js**: use shadcn/ui components (Radix patterns + Tailwind).
- **Leptos**: use `leptos-shadcn-ui` for equivalent primitives.
- **Remove DaisyUI** to reduce duplicate styling systems and ensure a single source of UI truth.

Note: the canonical shadcn/ui implementation is Tailwind-first, so Tailwind remains the styling substrate
for the Next.js apps. The Leptos side can consume the same design tokens (CSS variables) while rendering
the shadcn-style components with leptos-shadcn-ui.

## Shared UI strategy

### 1) Design tokens (single source of truth)

Define tokens (colors, radii, spacing, typography) once and consume them in both stacks.
Recommended approach:

- Add a token file (e.g. `docs/ui/tokens.json`) that defines the canonical palette.
- Map tokens into Tailwind config for Next.js.
- Export the same tokens as CSS variables for Leptos.

### 2) Shared component library (contract, not shared runtime)

We keep **one component spec** and **two implementations**:

- **Component contract**: props, states, and variants (documented once).
- **Next.js implementation**: shadcn/ui-based components.
- **Leptos implementation**: leptos-shadcn-ui-based components.

This avoids forcing a single runtime but keeps **behavior and styles consistent**.

### 3) Parity matrix

Create and maintain a small table (e.g. `docs/ui/parity-matrix.md`) with columns:

- Component name
- Next.js status
- Leptos status
- Notes (a11y, variants, missing states)

## Parallel development plan

1. **Design tokens first**
   - Lock tokens before building screens.
   - Review any token changes in both apps.

2. **Component parity in pairs**
   - Each component is implemented in both stacks before it is “accepted.”
   - Use the same variants and visual states (default, hover, focus, disabled, loading).

3. **Screen parity milestones**
   - Deliver screens in slices: Login → Users → Settings → Dashboard.
   - Each slice must ship in both apps together.

4. **Definition of done for parity**
   - Token usage matches.
   - Component variants match.
   - Layout spacing matches.
   - Accessibility checks (keyboard + focus states) align.

5. **Review checklist**
   - Compare screen snapshots across stacks.
   - Verify tokens only (no ad‑hoc colors/spacing).
   - Record any UI gaps and link to a follow-up plan to remove them.

## Storefront applicability

This strategy is **compatible with the storefronts** because both are already Tailwind/Leptos-based
and can consume the same design tokens and shadcn-style component contracts.

- `apps/next-frontend`: use shadcn/ui + Tailwind.
- `apps/storefront`: use leptos-shadcn-ui + CSS variables.

## Next steps

- Add a token definition file.
- Define a parity matrix starter list.
- Build the first shared components (Button, Input, Select, Card, Table).
- Migrate admin screens first, then storefront screens.
- Maintain a UI gap log and update it as Leptos libraries evolve.
- Track blocked Leptos library upgrades in the technical parity doc until upstream fixes are available.
