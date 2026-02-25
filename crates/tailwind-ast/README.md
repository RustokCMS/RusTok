# tailwind-ast (vendored)

Vendored fork of the `tailwind-ast` crate from [`cloud-shuttle/tailwind-rs`](https://github.com/cloud-shuttle/tailwind-rs).

See [Readme.md](./Readme.md) for original upstream documentation.

## Purpose

AST and parsing layer for Tailwind CSS class strings — tokenises class names into structured instruction trees consumed by `tailwind-css`.

## Responsibilities

- Lexes and parses Tailwind class name strings into an AST
- Provides modifier and variant parsing (responsive, state, arbitrary values)
- Used as the lowest layer of the Tailwind processing pipeline

## Entry Points

- `src/lib.rs` — parser entry points, AST node types

## Interactions

- **Consumed by**: `tailwind-css`
- **Depends on**: `tailwind-error`
- **Upstream**: https://github.com/cloud-shuttle/tailwind-rs (vendored)

## Maintenance

Synced via `scripts/tailwind/vendor_tailwind_rs.sh`. See [tailwind-rs README](../tailwind-rs/README.md).

## Links

- [Platform docs](../../docs/index.md)
- [tailwind-rs crate](../tailwind-rs/README.md)
- [tailwind-css crate](../tailwind-css/README.md)
