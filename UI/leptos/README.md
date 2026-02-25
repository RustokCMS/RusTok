# iu-leptos

Leptos (Rust/WASM) component library for the IU design system.

Provides the same component API as `UI/next/components/` but implemented in Leptos for use in `apps/admin` and `apps/storefront`.

## Purpose

- Implement IU design system components natively in Leptos
- Use shared CSS custom properties from `UI/tokens/base.css` (`--iu-*`)
- Expose the same prop contracts as the Next.js counterparts (see `UI/docs/api-contracts.md`)

## Responsibilities

- `Button`, `Input`, `Textarea`, `Select`, `Checkbox`, `Switch`, `Badge`, `Spinner` — base form/action primitives
- CSS-variable-based theming (no hardcoded Tailwind color classes)
- Re-exported by `crates/leptos-ui` which adds domain-specific wrappers (`Card`, `Label`, `Separator`)

## Entry Points

- `src/lib.rs` — public API (`pub use` all components)
- `src/types.rs` — shared enums: `ButtonVariant`, `BadgeVariant`, `Size`

## Component Index

| Component | File | Status |
|-----------|------|--------|
| `Button` | `src/button.rs` | ✅ |
| `Input` | `src/input.rs` | ✅ |
| `Textarea` | `src/textarea.rs` | ✅ |
| `Select` | `src/select.rs` | ✅ |
| `Checkbox` | `src/checkbox.rs` | ✅ |
| `Switch` | `src/switch.rs` | ✅ |
| `Badge` | `src/badge.rs` | ✅ |
| `Spinner` | `src/spinner.rs` | ✅ |

## Interactions

- **Consumed by**: `crates/leptos-ui` (re-exports), `apps/admin`, `apps/storefront`
- **Depends on**: `leptos` (workspace), `serde` (for derive on enums)
- **Tokens**: inherits `--iu-*` CSS variables — host app must import `UI/tokens/base.css`

## Links

- [IU API Contracts](../docs/api-contracts.md)
- [UI Tokens](../tokens/base.css)
- [leptos-ui crate](../../crates/leptos-ui/) — thin re-export wrapper
- [Platform docs](../../docs/index.md)
