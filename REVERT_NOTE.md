# Revert Note: Tailwind-rs Restored

## Date: February 11, 2026

## What Happened

Initially attempted to fix the `parcel_css` compilation issue by migrating from `tailwind-rs` to Node.js-based Tailwind CSS CLI.

## Why Reverted

**Architectural Mismatch**: 
- RusToK uses **Leptos** (Rust WASM framework), not Next.js or traditional JS frameworks
- The original implementation was designed for **native Rust/WASM** compilation
- Adding Node.js dependency contradicts the "Rust-native" philosophy of the project
- `tailwind-rs` was chosen specifically for WASM compatibility

## Current Status

✅ **Reverted all changes**:
- Restored `tailwind-rs` dependency in all `Cargo.toml` files
- Restored original `Trunk.toml` hooks using `tailwind-rs` command
- Removed `package.json`, `node_modules`, and npm-related files
- Removed Tailwind CLI documentation files
- Restored CODE_AUDIT_REPORT to original ⚠️ WORKAROUND status

## The Problem Remains

⚠️ Frontend apps (`apps/admin`, `apps/storefront`) are still blocked by:
- `parcel_css` v1.0.0-alpha.32 compilation errors
- API incompatibility issues

## Recommended Path Forward

The user will handle the `parcel_css` issue themselves, possibly by:
1. Forking and patching `tailwind-rs` with updated dependencies
2. Finding an alternative Rust-native Tailwind implementation
3. Using a custom CSS solution compatible with WASM
4. Waiting for upstream `tailwind-rs` or `parcel_css` fixes

## Lessons Learned

- Always verify the target framework (Leptos/WASM) before suggesting solutions
- Node.js-based solutions don't fit Rust-native WASM projects
- The original architecture choice (tailwind-rs) was intentional for WASM compatibility

## Files Reverted

- `Cargo.toml` - restored `tailwind-rs`
- `apps/admin/Cargo.toml` - restored `tailwind-rs`
- `apps/storefront/Cargo.toml` - restored `tailwind-rs`
- `apps/admin/Trunk.toml` - restored `tailwind-rs` hooks
- `apps/admin/tailwind.config.js` - restored original config
- `.gitignore` - removed npm-specific entries
- `README.md` - removed migration note
- `CODE_AUDIT_REPORT_2026-02-11.md` - restored original status

## Files Deleted

- `package.json`
- `package-lock.json`
- `node_modules/`
- `apps/admin/output.css`
- `apps/storefront/tailwind.config.js`
- `TAILWIND_CSS_SETUP.md`
- `TAILWIND_CSS_SOLUTION_PLAN.md`
- `TAILWIND_MIGRATION_SUMMARY.md`
- `РЕШЕНИЕ_ПРОБЛЕМЫ_TAILWIND.md`
- `SUMMARY_TAILWIND_FIX.txt`

## Current Repository State

Back to the state before the Tailwind CLI migration attempt, with:
- ✅ Backend compiles successfully
- ✅ TransactionalEventBus fixes applied
- ✅ IggyTransport `as_any()` fix applied
- ⚠️ Frontend apps disabled due to `parcel_css` issue (user will fix)
