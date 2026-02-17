# AGENTS

This repository is owned by the RusToK platform team and organized around domain modules.

## How to engage

- Review the domain module documentation before making changes.
- Use module owners (or the platform team) for approvals when cross-cutting concerns are involved.
- For architecture changes, capture decisions in `DECISIONS/` using an ADR.

## Ownership map

- **Platform foundation**: `crates/rustok-core`, `apps/server`, shared infra.
- **Domain modules**: `crates/rustok-*` (content, commerce, pages, blog, forum, index, etc.).
- **Frontends**: `apps/admin`, `apps/storefront`.
- **Operational tooling**: `scripts/`, `docker-compose*.yml`, `grafana/`, `prometheus/`.

Detailed module ownership and responsibilities should be captured under `docs/modules/`.
