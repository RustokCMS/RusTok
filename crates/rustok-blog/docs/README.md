# rustok-blog docs

В этой папке хранится документация модуля `crates/rustok-blog`.

## Documents

- [Implementation plan](./implementation-plan.md) — план развития модуля

## Модуль в картинке

```
┌─────────────────────────────────────────────────────────┐
│                    rustok-blog                          │
│                                                         │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │
│  │ PostService │  │  DTOs       │  │ StateMachine│     │
│  │             │  │             │  │             │     │
│  │ - create    │  │ - Create    │  │ Draft       │     │
│  │ - update    │  │ - Update    │  │ Published   │     │
│  │ - publish   │  │ - Response  │  │ Archived    │     │
│  │ - archive   │  │ - Query     │  │ Comment     │     │
│  │ - delete    │  │             │  │             │     │
│  └─────────────┘  └─────────────┘  └─────────────┘     │
│         │                                     │         │
│         │                                     │         │
│         ▼                                     ▼         │
│  ┌─────────────┐                      ┌─────────────┐   │
│  │  BlogError  │                      │ Permissions │   │
│  │             │                      │             │   │
│  │ RichError   │                      │ posts:*     │   │
│  │ compatible  │                      │ comments:*  │   │
│  └─────────────┘                      │ categories:*│   │
│                                       │ tags:*      │   │
│                                       └─────────────┘   │
└─────────────────────────────────────────────────────────┘
                          │
                          │ Uses
                          ▼
┌─────────────────────────────────────────────────────────┐
│                  rustok-content                         │
│                                                         │
│  Nodes, Bodies, Translations (storage layer)           │
└─────────────────────────────────────────────────────────┘
```

## Ключевые решения

### Wrapper Module Pattern
Blog не создаёт собственные таблицы, а использует таблицы content-модуля с `kind = "post"`. Это:
- Уменьшает дублирование схемы
- Обеспечивает консистентность данных
- Позволяет использовать общий функционал (версионирование, локализация)

### Type-Safe State Machine
Статусы постов реализованы как типобезопасная state machine:
- Невалидные переходы невозможны на уровне компилятора
- Каждый статус содержит специфичные данные (published_at, reason)
- Легко тестировать и документировать

### Rich Errors
Все ошибки конвертируются в `RichError`:
- Понятные сообщения для пользователей
- Детальная информация для разработчиков
- Коды ошибок для автоматической обработки


## Roadmap / status

Update 2026-03-29: canonical blog tag vocabulary now lives in shared
`rustok-taxonomy`; `rustok-blog` keeps `blog_post_tags` as its typed
relation table and reuses matching global terms on post write paths.

Краткая синхронизация с `implementation-plan.md`:

- ✅ `PostService` и `CommentService` реализованы и покрыты unit + частью integration сценариев.
- ✅ GraphQL и REST адаптеры модуля перенесены в `crates/rustok-blog`; `apps/server` держит только тонкий shim/composition-root слой.
- ✅ Publishable Leptos UI пакеты `crates/rustok-blog/admin` и `crates/rustok-blog/storefront` теперь тоже живые: `BlogAdmin` делает list/create/edit/update/publish/archive/delete через модульный GraphQL, а `BlogView` читает published post по `?slug=` и каталог публикаций через тот же контракт.
- ✅ Comment runtime contract теперь приходит из `rustok-comments`: create-path использует generic comments storage, thread status `closed` блокирует новые записи, а moderation statuses остаются за comment-domain.
- ✅ `CategoryService` (`src/services/category.rs`) реализован и использует blog-owned category tables.
- ✅ `TagService` (`src/services/tag.rs`) реализован поверх shared `rustok-taxonomy`; canonical term storage живёт в taxonomy, а `blog_post_tags` остаётся blog-owned relation.
- ✅ Integration-тесты уже покрывают post lifecycle, category/tag CRUD и taxonomy-backed tag sync, включая reuse глобального term.

## Композиция API

- GraphQL entry points: `graphql::BlogQuery`, `graphql::BlogMutation`.
- HTTP entry point: `controllers::routes()`.
- Общие transport-контракты (`AuthContext`, `TenantContext`, `RequestContext`, GraphQL helpers) модуль получает из `rustok-api`, а не из `apps/server`.
- Author presentation contract больше не должен жить на прямой завязке к `users`: GraphQL payloads `GqlPost`/`GqlPostListItem` теперь отдают `author_profile`, который резолвится через `rustok-profiles::ProfilesReader`, а в shared server runtime автоматически использует `ProfileSummaryLoader` как request-scoped cache при его наличии.
- Для Leptos UI модуль теперь также публикует `admin/` и `storefront/` sub-crate-ы, которые используют тот же GraphQL contract без knowledge о `apps/admin` или `apps/storefront`.

## Channel-aware pilot

`rustok-blog` стал вторым pilot consumer для `rustok-channel`, чтобы сравнить один и тот же runtime-паттерн уже на двух доменах.

- public GraphQL read-path (`postBySlug`, `posts`, а также `post` без auth) смотрит на `RequestContext.channel_id`;
- если для текущего канала есть `channel_module_bindings` c `module_slug = "blog"` и `is_enabled = false`, модульный read-path возвращает `MODULE_NOT_ENABLED`;
- если binding отсутствует, в v0 действует permissive fallback: `blog` считается доступным;
- authenticated/admin flows этот channel gate не блокирует, чтобы не ломать операторские сценарии и admin UX.
- поверх этого уже добавлен второй publication-level proof point через metadata-based `channelSlugs` allowlist на постах;
- allowlist хранится в metadata по пути `channel_visibility.allowed_channel_slugs`;
- unauthenticated public read-path показывает только опубликованные посты, видимые для текущего `channel_slug`;
- authenticated/admin flows publication-level visibility сознательно bypass-ят, чтобы не ломать редакторские сценарии;
- в v0 эта модель остаётся metadata-based: отдельная relation/table для post-to-channel отложена до появления требований, которые нельзя закрыть request-time filtering.

Update 2026-03-29: publication-level `channelSlugs` visibility now uses typed
`blog_post_channel_visibility` persistence and DB-side filtering on public
list/read paths. Any older bullets above describing metadata-only storage are
superseded by the live code.

## Event contracts

- [Event flow contract (central)](../../../docs/architecture/event-flow-contract.md)

## Rich-text contract

- Blog/Forum backend принимает `markdown` и `rt_json_v1` payload; для `rt_json_v1` выполняется обязательные `schema validation + sanitize` на сервере.
- Клиентские валидаторы считаются advisory-only и не являются источником истины.
- Детали спецификации: [docs/standards/rt-json-v1.md](../../../docs/standards/rt-json-v1.md).
- **Response contract (read path):** backend всегда возвращает `*_format` (`body_format`/`content_format`) и нормализованное поле `content_json` для `rt_json_v1`; при `markdown` `content_json = null`, а текст остаётся в `body/content` для обратной совместимости.
- Для миграции legacy markdown-записей используйте tenant-scoped job `cargo run -p rustok-server --bin migrate_legacy_richtext -- --tenant-id=<uuid> [--dry-run]`; job идемпотентный (checkpoint + retry) и безопасен для поэтапного rollout tenant-by-tenant.


## Leptos UI packages

- `crates/rustok-blog/admin` теперь содержит module-owned root admin page `BlogAdmin` с базовым CRUD-потоком публикаций через GraphQL (`list/create/edit/update/publish/archive/delete`).
- `crates/rustok-blog/storefront` теперь содержит module-owned storefront surface `BlogView`, который читает published post по `?slug=` и каталог публикаций через `postBySlug` + `posts`.
- Для обоих пакетов tenant/route context остаётся generic: tenant берётся сервером из `TenantContext`, а storefront route state приходит через `rustok-api::UiRouteContext`.
