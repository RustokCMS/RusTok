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
# AI Agent Rules

Эти правила обязательны для любых автоматизированных агентов, работающих в репозитории RusToK.

## Основные правила

1. Всегда начинайте работу с изучения `docs/DOCS_MAP.md`.
2. Не создавайте новый документ, если есть подходящий — расширяйте существующий.
3. При изменениях **архитектуры, API, событий, модулей, tenancy, маршрутизации, UI контрактов, observability**:
   - обновите релевантную документацию,
   - обновите записи в `docs/DOCS_MAP.md`.
4. Устаревшие документы помечайте как `deprecated` или `archived` и указывайте замену.
5. Документация должна отражать реальное состояние кода и системы.
