# rustok-forum docs

В этой папке хранится документация модуля `crates/rustok-forum`.

## Documents

- [Implementation plan](./implementation-plan.md)
- [Admin UI package](../admin/README.md)
- [Storefront UI package](../storefront/README.md)

## Status

- Split-track `forum off rustok-content` закрыт: persistence, locale contract и
  slug/locale ADR теперь совпадают с live code.

- ✅ GraphQL и REST адаптеры модуля перенесены в `crates/rustok-forum`; `apps/server` держит только тонкий shim/composition-root слой.
- ✅ Общие transport-контракты (`AuthContext`, `TenantContext`, `RequestContext`, GraphQL helpers) модуль получает из `rustok-api`.
- ✅ Author presentation для topics/replies теперь идёт через `rustok-profiles::ProfilesReader`: GraphQL payloads `GqlForumTopic` и `GqlForumReply` отдают `author_profile` рядом с `author_id`, а в shared server runtime используют `ProfileSummaryLoader` как request-scoped cache там, где он зарегистрирован.
- ✅ `forum_solutions` добавлен как forum-owned Q&A capability: topic read-path теперь отдаёт `solution_reply_id`, reply read-path — `is_solution`, а mark/clear идут через module-owned moderation/service layer.
- ✅ `forum_topic_votes` / `forum_reply_votes` подняты как forum-owned capability: topic/reply read-path теперь отдают `vote_score` и `current_user_vote`, а GraphQL/REST могут set/clear vote через module-owned service layer.
- ✅ `forum_subscriptions` поднят как forum-owned capability: category/topic read-path теперь отдают viewer-specific `is_subscribed`, а GraphQL/REST могут subscribe/unsubscribe без отдельного shared notification module.
- ✅ `forum_user_stats` поднят как forum-owned capability: table `forum_user_stats` хранит tenant-scoped `topic_count/reply_count/solution_count`, write-path держит их в sync, а GraphQL/REST отдают отдельный user-stats read-path.
- ✅ Publishable Leptos admin UI для forum теперь живёт в `crates/rustok-forum/admin/`; host admin подключает пакет через manifest-driven `build.rs`.
- ✅ Publishable Leptos storefront UI для forum теперь живёт в `crates/rustok-forum/storefront/`; host storefront подключает пакет через manifest-driven `build.rs`, а public GraphQL read-path отдаёт categories, topic feed, thread detail и replies для `/modules/forum`.

- ✅ Topic tags now use forum-owned `forum_topic_tags` over `rustok-taxonomy`: forum keeps attachment ownership, while shared/global and forum-local term identity lives in the taxonomy dictionary.

## Event contracts

- [Event flow contract (central)](../../../docs/architecture/event-flow-contract.md)

## Rich-text contract

- Blog/Forum backend принимает `markdown` и `rt_json_v1` payload; для `rt_json_v1` выполняется обязательные `schema validation + sanitize` на сервере.
- Клиентские валидаторы считаются advisory-only и не являются источником истины.
- Детали спецификации: [docs/standards/rt-json-v1.md](../../../docs/standards/rt-json-v1.md).
- **Response contract (read path):** backend всегда возвращает `*_format` (`body_format`/`content_format`) и нормализованное поле `content_json` для `rt_json_v1`; при `markdown` `content_json = null`, а текст остаётся в `body/content` для обратной совместимости.
- Для миграции legacy markdown-записей используйте tenant-scoped job `cargo run -p rustok-server --bin migrate_legacy_richtext -- --tenant-id=<uuid> [--dry-run]`; job идемпотентный (checkpoint + retry) и безопасен для поэтапного rollout tenant-by-tenant.

## Наблюдаемость

- public REST/GraphQL read-path форума уже пишет `rustok_read_path_query_duration_seconds`
  и `rustok_read_path_*` budgets для categories, topic feeds и replies;
- accepted solution сейчас отражается в обычном topic/reply read-path и не требует
  отдельного read-model; если solution-state начнёт участвовать в ranking/feed logic,
  это нужно будет выделить в отдельный observability slice;
- voting пока считается inline на topic/reply read-path для page-sized выборок; если
  vote-score начнёт участвовать в глобальном ranking/feed materialization, это нужно
  будет вынести в отдельный read-model и отдельные latency/saturation метрики;
- subscriptions сейчас тоже считаются inline как viewer-specific flag на category/topic
  read-path; если появится отдельный notification dispatcher или digest pipeline,
  это нужно будет отделить от read contract и зафиксировать в отдельном observability slice;
- user-stats сейчас обновляются синхронно в transaction-bound write-path; если позже
  статистика начнёт включать тяжёлые derived metrics, это нужно будет выносить в
  отдельный projection/read-model слой с собственными freshness-метриками;
- storefront visibility/permission filtering отражается в effective
  `returned_items`, поэтому оператор видит не только размер сырой выборки, но и
  фактический объём видимой выдачи;
- moderation/read-model incidents следует сначала разделять на три класса:
  `forum_topics/forum_replies` permission denial, visibility filtering и
  storage/query latency;
- module-level write-path counters для forum пока не выделены отдельно, поэтому
  operational baseline сейчас опирается на read-path telemetry, сервисные тесты
  и event-flow contract.
