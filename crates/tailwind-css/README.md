# tailwind-css (vendored)

Vendored fork of the `tailwind-css` crate from [`cloud-shuttle/tailwind-rs`](https://github.com/cloud-shuttle/tailwind-rs).

See [Readme.md](./Readme.md) for original upstream documentation.

## Purpose

Core Tailwind CSS JIT/AOT interpreter — parses Tailwind class strings and emits CSS property trees.

## Responsibilities

- Resolves Tailwind utility classes to CSS `(property, value)` pairs
- Supports compile-time (`compile_time` feature) and runtime resolution
- Provides the `TailwindBuilder` type used downstream by `tailwind-rs`

## Entry Points

- `src/lib.rs` — `TailwindBuilder`, `TailwindInstruction`, class parser

## Interactions

- **Consumed by**: `tailwind-rs` (HTML-mode pipeline)
- **Depends on**: `tailwind-ast`, `tailwind-error`
- **Upstream**: https://github.com/cloud-shuttle/tailwind-rs (vendored)

## Maintenance

Synced via `scripts/tailwind/vendor_tailwind_rs.sh`. See [tailwind-rs README](../tailwind-rs/README.md).

## Links

- [Platform docs](../../docs/index.md)
- [tailwind-rs crate](../tailwind-rs/README.md)
- [tailwind-ast crate](../tailwind-ast/README.md)
