# Storefront (Leptos SSR + Next.js)

RusToK storefront ships in two flavors:
- **Leptos SSR** (`apps/storefront`) for a Rust-first, SSR application styled with Tailwind.
- **Next.js App Router** (`apps/next-frontend`) for teams that want a React-based storefront.

Both variants start with a minimal landing layout that can be extended with
product listings, content blocks, and checkout flows.

## Run locally

```bash
# Leptos SSR
cargo run -p rustok-storefront

# Next.js storefront
cd apps/next-frontend
npm install
npm run dev
```

The Leptos SSR server listens on `http://localhost:3100`. The Next.js app runs on
`http://localhost:3000` by default.

## Tailwind styles

The Leptos storefront ships with Tailwind-only styling. The CSS pipeline uses
`tailwind-rs` for the WASM-first, type-safe build. For offline or customized
themes, build the CSS bundle:

```bash
cd apps/storefront
npm install
npm run build:css
```

This writes `apps/storefront/static/app.css`, which the SSR server serves from
`/assets/app.css`.

## Localization

The storefront currently supports English and Russian strings. Switch language
with the `lang` query parameter:

- English: `http://localhost:3100?lang=en`
- Russian: `http://localhost:3100?lang=ru`

Add more locales by extending the `locale_strings` mapping in
`apps/storefront/src/main.rs`.

This is an alpha version and requires clarification. Be careful, there may be errors in the text. So that no one thinks that this is an immutable rule.
