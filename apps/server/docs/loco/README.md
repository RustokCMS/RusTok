# Loco.rs docs index for RusToK

Этот документ — **навигационный индекс** по Loco-документации в репозитории.

## ⚠️ Для AI-агентов: читать в первую очередь

Если вы меняете `apps/server/**`, сначала проверьте:

1. Этот файл (`apps/server/docs/loco/README.md`);
2. `apps/server/docs/loco/changes.md`;
3. `apps/server/docs/library-stack.md` (основные библиотеки сервера и роли);
4. `apps/server/docs/loco/upstream/VERSION` (актуальность snapshot);
5. Текущие паттерны в `apps/server/src/**` и `apps/server/migration/**`.

Короткое правило: **реальный код в `apps/server` важнее абстрактных рекомендаций из интернета**.

## Что это

1. [Upstream Loco.rs snapshot (`./upstream/`)](./upstream/)
   - Это pinned-копия официальной документации Loco.rs.
   - Версия источника зафиксирована в [`./upstream/VERSION`](./upstream/VERSION).

> **Правило для AI-агентов и контрибьюторов:** при вопросах по Loco **сначала сверяться с `upstream/`**, и только потом с локальными заметками ниже.

## Repo-specific notes (только отличия RusToK от default Loco)

- Серверная реализация живёт в `apps/server` и может вводить проектные ограничения поверх дефолтных возможностей Loco.
- При проектировании изменений приоритет у реального кода и текущих модулей (`app.rs`, `controllers/`, `models/`, `migration/`).
- Краткие изменения локальных практик ведутся в [`changes.md`](./changes.md).

## Обновление upstream snapshot

```bash
scripts/docs/sync_loco_docs.sh
```

## Что важно для AI-агентов

- Loco.rs уже используется как backend framework — не предлагать замену фреймворка для базовых задач.
- Для auth, permissions, migrations и контроллеров опираться на текущие паттерны проекта, а не абстрактные "универсальные" рецепты.
- Если в коде есть расхождения между общим guidance и реальной реализацией — приоритет у реального кода в `apps/server`.

## Как поддерживать "свежесть" этого контекста

- При изменении server-архитектуры обновлять этот файл в том же PR.
- При крупных изменениях Loco-слоя добавлять короткие заметки в `apps/server/docs/loco/changes.md`.

## Upstream snapshot freshness

`apps/server/docs/loco/upstream/VERSION` stores snapshot metadata for upstream Loco references.

- `make docs-check-loco` validates that metadata exists and enforces freshness policy:
  - `>30` days old: CI warning;
  - `>60` days old: CI failure.
- `make docs-sync-loco` refreshes snapshot metadata date before opening a PR.

## Как удалить Loco-документацию и автоматизацию (если временная мера больше не нужна)

Удаляйте это одним PR, чтобы не оставлять «битые» CI-проверки:

1. Удалить папку документации:
   - `apps/server/docs/loco/` (включая `upstream/VERSION`).
2. Удалить скрипт автоматизации:
   - `scripts/loco_upstream_snapshot.py`.
3. Удалить make-цели:
   - `docs-sync-loco` и `docs-check-loco` из `Makefile`.
4. Удалить CI-job:
   - `loco-docs-snapshot` из `.github/workflows/ci.yml`;
   - убрать его из `ci-success.needs` и из финального условия проверки.
5. Удалить пункт из PR-шаблона:
   - checkbox про актуальность `apps/server/docs/loco/upstream`.

Минимальная проверка после удаления:

```bash
cargo check --workspace --all-targets --all-features
```

и убедиться, что workflow CI проходит без `loco-docs-snapshot`.
