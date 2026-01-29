# Storefront (Leptos SSR)

RusToK storefront is an SSR-first Leptos app styled with Tailwind + DaisyUI. It
ships with a minimal landing layout that can be extended with product listings,
content blocks, and checkout flows.

## Run locally

```bash
cargo run -p rustok-storefront
```

The server listens on `http://localhost:3100`.

## Tailwind + DaisyUI styles

By default the storefront uses Tailwind CDN plus the DaisyUI stylesheet for quick
local previews. For offline or customized themes, build the CSS bundle:

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
