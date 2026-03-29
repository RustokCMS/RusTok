# Документация `rustok-content`

В этой папке хранится документация модуля `crates/rustok-content`.

## Документы

- [Implementation plan](./implementation-plan.md)

## Event contracts

- [Event flow contract (central)](../../../docs/architecture/event-flow-contract.md)

## Runtime role

- `rustok-content` больше не публикует product GraphQL/REST/admin/storefront adapters.
- `apps/server` больше не реэкспортирует content transport как публичный API.
- Модуль остаётся в runtime как shared helper слой с собственными migrations и orchestration state.

## Orchestration

- `ContentOrchestrationService` теперь является port-based orchestration core.
- Сервис валидирует RBAC, idempotency key и unsafe input, пишет
  `content_orchestration_operations` / `content_orchestration_audit_logs`
  и публикует доменные события.
- Сервис также владеет typed canonical state:
  `content_canonical_urls` и `content_url_aliases`.
- Доменные conversion-операции (`promote_topic_to_post`, `demote_post_to_topic`,
  `split_topic`, `merge_topics`) делегируются через `ContentOrchestrationBridge`.
- `rustok-content` больше не переносит shared `node`-children между родителями и не
  знает о `blog/forum/comments` persistence напрямую.
- `apps/server` bridge теперь возвращает явные URL-mutations, а orchestration core
  фиксирует canonical/alias mapping атомарно в той же транзакции, где пишет
  audit/idempotency и outbox-события.
- Runtime enforcement RBAC в `ensure_scope(...)` требует следующие permissions:
  - `promote_topic_to_post`: `forum_topics:moderate` + `blog_posts:create`
  - `demote_post_to_topic`: `blog_posts:moderate` + `forum_topics:create`
  - `split_topic`: `forum_topics:moderate`
  - `merge_topics`: `forum_topics:moderate`

## Conversion mode policy

- Текущий live contract поддерживает только `move_and_redirect`.
- `copy` не входит в runtime contract, потому что создаёт два конкурирующих canonical source.
- plain `move` без redirect тоже не поддерживается: legacy URL обязан становиться alias-ом с redirect на новую canonical цель.

## Shared rich-text contract

- `blog` и `forum` используют `prepare_content_payload(...)` как единый write-path для `markdown` и `rt_json_v1`.
- `comments` используют тот же allowlist body-format через shared validator `validate_body_format(...)` из `rustok-content`.
- Канонический rich-text формат для publishable content surfaces — `rt_json_v1`.
- `rt_json` остаётся только compatibility alias и не должен становиться новым source format в storage-моделях.
- Пустой body допустим только для rich-text payload (`rt_json_v1` / `rt_json`); для `markdown` и других plain/textual formats требуется непустое содержимое.

## Conversion mapping rules

- authorship сохраняется: author/source owner переносится в target entity, а author каждого comment/reply остаётся привязан к исходному сообщению;
- timestamps сохраняются: `created_at` и исходный порядок сообщений не пересоздаются произвольно при conversion;
- body и body_format переносятся без downgrade в другой формат;
- moderation mapping явный:
  - `forum_reply.approved -> comment.approved`
  - `forum_reply.pending -> comment.pending`
  - `forum_reply.flagged -> comment.spam`
  - `forum_reply.hidden|rejected -> comment.trash`
  - `comment.approved -> forum_reply.approved`
  - `comment.pending -> forum_reply.pending`
  - `comment.spam -> forum_reply.flagged`
  - `comment.trash -> forum_reply.hidden|deleted` в зависимости от `deleted_at`
- soft-delete semantics сохраняются через явный mapping статусов, а не через silent drop записей при переносе.

## Canonical URL policy после конвертаций

### 1. Канонический источник после `topic ↔ post` конвертации

- Каноническим источником всегда считается текущая активная сущность после orchestration-операции.
- При `promote_topic_to_post` canonical-представлением становится `post`.
- При `demote_post_to_topic` canonical-представлением становится `topic`.
- Исходная сущность после конвертации трактуется как legacy alias и не должна оставаться
  самостоятельной индексируемой страницей.

### 2. Redirect + canonical tags

- Для всех legacy URL после конвертации обязан отдаваться redirect на canonical URL.
- На canonical-странице выставляется `<link rel="canonical">` на self URL.
- На временно доступных неканонических представлениях canonical tag должен указывать на canonical URL,
  а страница должна быть `noindex`.
- `CanonicalUrlService` резолвит alias/canonical route по тому же locale fallback-контракту, что и остальные
  content read-path'ы: `requested -> platform fallback en -> first available`.

### 3. Защита от дублей

- Нельзя одновременно держать индексируемыми старый `topic` URL и новый `post` URL для одного и того же контента.
- В любой момент времени для одной логической публикации допускается ровно один canonical URL на locale.
- Все alias/исторические URL фиксируются как redirect-источники и исключаются из самостоятельной индексации.

### 4. Правила slug collision

- Уникальность slug обеспечивается в рамках `tenant + locale`.
- Если новый canonical slug конфликтует с активным slug другой сущности, операция не должна публиковаться до разрешения конфликта.
- Разрешение collision должно быть детерминированным, а redirect/canonical mapping должны пересчитываться атомарно.

### 5. Locale-specific slug migration

- Миграция slug выполняется по локалям независимо: canonical URL определяется отдельно для каждой locale-версии.
- Если в locale нет собственного slug/перевода, применяется fallback locale по tenant policy, но canonical URL всё равно должен быть единственным для resolved locale.
- При появлении локализованного slug позже старый fallback URL становится alias с redirect на новый locale-specific canonical URL.

### 6. Связь с индексатором (`rustok-index`)

- Любая смена canonical URL должна порождать reindex canonical-цели и purge/deindex устаревших URL aliases.
- Порядок обязателен для at-least-once delivery: сначала фиксируется новый canonical mapping, затем публикуются reindex/purge события через outbox.
- Обработчики индексатора должны быть идемпотентны.
