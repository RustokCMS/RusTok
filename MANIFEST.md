# Repository Manifest

RusToK is a Rust monorepo for an event-driven, modular headless CMS + e-commerce platform.

## Canonical manifests

- **Product manifest**: [`RUSTOK_MANIFEST.md`](./RUSTOK_MANIFEST.md)
- **Module registry**: [`modules.toml`](./modules.toml) (with example in `modules.toml.example`)

## Documentation entry points

- [`docs/index.md`](./docs/index.md) for structured navigation
- [`README.md`](./README.md) and [`README.ru.md`](./README.ru.md) for onboarding
- [`QUICKSTART.md`](./QUICKSTART.md) for local setup

## Repo layout

- `apps/` — server and UI applications
- `crates/` — domain modules and shared libraries
- `docs/` — architecture, standards, and module documentation
- `scripts/`, `docker-compose*.yml` — operational and dev tooling
