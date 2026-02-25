# tailwind-rs (vendored)

Vendored fork of [`cloud-shuttle/tailwind-rs`](https://github.com/cloud-shuttle/tailwind-rs).

See [Readme.md](./Readme.md) for full vendoring rationale and maintenance instructions.

## Purpose

HTML-mode Tailwind CSS class translator used by the `apps/admin` Trunk build pipeline.

## Responsibilities

- Parses HTML and translates Tailwind utility class strings into optimised CSS
- Provides Trunk integration (called during WASM build of `apps/admin`)
- Delegates to `tailwind-css` and `tailwind-ast` for the core parsing/instruction pipeline

## Entry Points

- `src/lib.rs` — top-level re-exports from `tailwind-css` with the `html` feature gate

## Interactions

- **Consumed by**: `apps/admin` build (via Trunk)
- **Depends on**: `tailwind-css`, `tailwind-ast`, `tailwind-error`, `parcel_css`, `tl`
- **Upstream**: https://github.com/cloud-shuttle/tailwind-rs (vendored, not a git submodule)

## Maintenance

When updating, run `scripts/tailwind/vendor_tailwind_rs.sh` to re-sync from upstream, then reapply local RusToK patches. See [Readme.md](./Readme.md) for details.

## Links

- [Platform docs](../../docs/index.md)
- [tailwind-css crate](../tailwind-css/README.md)
- [tailwind-ast crate](../tailwind-ast/README.md)
