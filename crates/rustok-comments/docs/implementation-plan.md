# План реализации `rustok-comments`

Этот документ фиксирует локальный roadmap модуля `rustok-comments`.

## Этап 1. Module foundation

- [x] добавить crate, `CommentsModule`, permissions и module manifest;
- [x] подключить модуль в workspace, `modules.toml`, server feature wiring и central docs;
- [x] зафиксировать локальную storage/API стратегию внутри module docs.

## Этап 2. Storage boundary

- [x] спроектировать таблицы `comment_threads`, `comments`, `comment_bodies`;
- [x] добавить module-owned migrations;
- [x] ввести entities/repositories и базовый `CommentService`.

### Target schema

- `comment_threads`
  - thread ownership per `(tenant_id, target_type, target_id)`
  - typed `status`, `comment_count`, `last_commented_at`
- `comments`
  - typed `thread_id`, `author_id`, `parent_comment_id`, `status`, `position`
  - no reuse of forum reply storage
- `comment_bodies`
  - locale-aware body storage with explicit `body_format`
  - canonical support for shared rich-text contracts from `rustok-content`

### Required indexes and constraints

- unique `(tenant_id, target_type, target_id)` on `comment_threads`
- unique `(comment_id, locale)` on `comment_bodies`
- ordered list indexes on `(thread_id, position)` and `(thread_id, created_at)`

## Этап 3. Domain contracts

- [x] определить target binding contract для blog/pages;
- [ ] определить moderation/status contract для comment-domain;
- [x] свести comment body к shared rich-text contract.

## Этап 4. Integrations

- [x] перевести `rustok-blog` на `rustok-comments`;
- [ ] определить интеграцию `rustok-pages` с `rustok-comments`, если она нужна продуктово;
- [x] добавить transport adapters в `apps/server`.

## Этап 5. Orchestration compatibility

- [x] реализовать mapping между `blog comments` и `forum replies` через `rustok-content`;
- [x] покрыть conversion flows end-to-end тестами после появления orchestration service.

## Текущее состояние

- `rustok-comments` — больше не scaffold, а live storage-owner для generic comments;
- `rustok-blog` уже использует модуль в production read/write path;
- дальнейший scope модуля теперь связан не со split, а с расширением moderation и
  product-level integrations.
