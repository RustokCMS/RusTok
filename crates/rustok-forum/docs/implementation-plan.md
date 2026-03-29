# rustok-forum module implementation plan (`rustok-forum`)

## Scope and objective

This document captures the forum-local implementation plan for `rustok-forum`
in RusToK and serves as the source of truth for rollout sequencing inside
`crates/rustok-forum`.

Primary objective: evolve `rustok-forum` in small, testable increments while
preserving clarity of the forum bounded context.

## Target architecture

- `rustok-forum` remains focused on its bounded context and public crate API.
- Integrations with other modules go through stable interfaces in `rustok-core`
  or explicit orchestration flows.
- Forum-specific behavior should live in forum-owned services and forum-owned
  persistence, not in generic shared storage metadata.
- Observability and operability requirements are part of delivery readiness.

## Current boundary

The cross-module storage split is complete:

- `rustok-forum` owns forum categories, topics, topic translations, replies,
  reply bodies, channel-access persistence, and topic-tag attachment persistence;
- `rustok-content` remains only as a shared helper/orchestration dependency;
- `rustok-taxonomy` is now the shared dictionary behind forum tags, while
  `forum_topic_tags` remains forum-owned storage;
- topic ↔ post conversion compatibility now lives in orchestration and server
  runtime adapters, not in shared storage.
- locale metadata is aligned on detail and list surfaces through shared
  `rustok-content` helpers;
- forum slug/locale semantics are now fixed by ADR
  `2026-03-29-forum-slug-locale-contract.md`.

This document intentionally keeps only forum-local implementation details and
future forum backlog.

## Delivery phases

### Phase 0 - Foundation (done)

- [x] Baseline crate/module structure is in place.
- [x] Base docs and registry presence are established.
- [x] Core compile-time integration with the workspace is available.

### Phase 1 - Contract hardening (done)

- [x] Freeze public API expectations for the current module surface.
  - Public surface: `CategoryService`, `TopicService`, `ReplyService`,
    `ModerationService` with CRUD operations.
  - `ModerationService` extended with topic operations: `pin_topic`,
    `unpin_topic`, `lock_topic`, `unlock_topic`, `close_topic`, `archive_topic`.
- [x] Align error/validation conventions with platform guidance.
  - Empty title/body/content/name/slug in `create` methods return
    `ForumError::Validation`.
  - Error types follow platform `thiserror` conventions.
- [x] Expand automated tests around core invariants and boundary behavior.
  - 9 inline lib tests for `node_to_topic`, `node_to_category`,
    `node_to_reply` mapping logic.
  - 15 pure unit tests in `tests/unit.rs`: constants, error display,
    DTO serde defaults.
  - 2 module contract tests in `tests/module.rs`: metadata and migrations list.
  - Integration test scaffold in `tests/integration.rs` (ignored, requires DB).

### Phase 2 - Bug fixes and i18n hardening (done)

- [x] Fix P0: `author_id` is now propagated from `SecurityContext` to
  `CreateNodeInput` in `TopicService` and `ReplyService`.
- [x] Fix P0: `ReplyService::create` now validates topic status before creating
  a reply.
  - Returns `ForumError::TopicClosed` if topic status is `closed`.
  - Returns `ForumError::TopicArchived` if topic status is `archived`.
- [x] Fix P0: `TopicService::list` and `CategoryService::list` now fetch full
  node data to populate metadata fields.
  - `is_pinned`, `is_locked`, `forum_status`, `reply_count`, `category_id`,
    `slug` are correctly populated from node metadata.
  - `icon`, `color`, `topic_count`, `reply_count` in `CategoryService::list`
    are correctly populated.
- [x] Fix P1: `locale.rs` module with `resolve_translation` / `resolve_body` /
  `available_locales` helpers.
  - Fallback chain: `requested -> explicit fallback -> "en" -> first available`.
  - All `node_to_*` mappers migrated to use the new helpers.
- [x] Fix P1: `requested_locale`, `effective_locale`, and `available_locales`
  are aligned on `TopicResponse`, `TopicListItem`, `CategoryResponse`, and
  `CategoryListItem`.
- [x] Fix P1: `effective_locale` added to `ReplyResponse` and `ReplyListItem`.
- [x] Fix P1: `author_id: Option<Uuid>` added to `TopicResponse`,
  `TopicListItem`, `ReplyResponse`, `ReplyListItem`.
- [x] Fix P1: `slug: Option<String>` added to `CreateTopicInput`; `slug`
  included in node translation.
- [x] Fix P1: `ListRepliesFilter` (new DTO) replaces hard-coded
  `per_page: 200` in `list_for_topic`. Signature changed to accept filter.
- [x] Fix P1: Forum-specific `DomainEvent` variants added to `rustok-core`.
  - `ForumTopicCreated`, `ForumTopicReplied`, `ForumTopicStatusChanged`,
    `ForumTopicPinned`, `ForumReplyStatusChanged`.
  - All variants include `event_type()`, `schema_version()`, `validate()`, and
    `affects_index()` support.
- [x] Fix P1: `TopicService::create` publishes `ForumTopicCreated` after node
  creation.
- [x] Fix P1: `ReplyService::create` publishes `ForumTopicReplied` after node
  creation.
- [x] Fix P1: `ModerationService` publishes `ForumTopicStatusChanged`,
  `ForumTopicPinned`, `ForumReplyStatusChanged`.
- [x] Fix P1: `ModerationService` methods for `approve_reply`, `reject_reply`,
  `hide_reply`, `pin_topic`, `unpin_topic`, `close_topic`, `archive_topic` now
  accept `tenant_id: Uuid`.

### Phase 3 - Forum-local productionization after split (done)

Forum-owned persistence baseline is now in place:

- forum categories, topics, topic translations, replies, reply bodies, and
  channel access no longer use `rustok-content::NodeService` as the primary
  storage backend;
- `rustok-content` remains only as a shared helper/orchestration dependency
  for locale and rich-text contracts;
- the remaining forum-local backlog is no longer about shared-storage cutover or
  multilingual contract cleanup; those are complete.

Forum-local backlog that remains relevant after the split:

- [x] `forum_topic_votes` / `forum_reply_votes` - voting tables.
  - [x] module-owned vote relations with `{-1, +1}` contract and cascade cleanup.
  - [x] topic/reply read-paths now expose aggregate `vote_score` plus
    viewer-specific `current_user_vote`.
  - [x] GraphQL + REST transport can set/clear topic and reply votes through
    forum-owned service layer.
- [x] `forum_solutions` - Q&A solution marking.
  - [x] module-owned `forum_solutions` relation table with topic/reply FK and
    cascade cleanup.
  - [x] topic read-path now exposes `solution_reply_id`; reply read-path exposes
    `is_solution`.
  - [x] GraphQL + REST transport can mark/clear accepted solution through
    forum-owned moderation/service layer.
- [x] `forum_subscriptions` - per-category/topic notification subscriptions.
  - [x] module-owned category/topic subscription relations with cascade cleanup.
  - [x] category/topic read-paths now expose viewer-specific `is_subscribed`.
  - [x] GraphQL + REST transport can subscribe/unsubscribe category and topic
    watches through forum-owned service layer.
- [x] `forum_user_stats` - per-user forum statistics.
  - [x] module-owned `forum_user_stats` table with tenant-scoped counters per user.
  - [x] topic/reply create-delete flows and solution mark/clear now keep
    `topic_count`, `reply_count`, and `solution_count` in sync.
  - [x] GraphQL + REST expose a dedicated per-user stats read-path.
- [x] Full forum-specific RBAC enforcement in moderation/runtime flows.
  - [x] Category/topic/reply services now re-validate forum permissions in the
    service layer.
  - [x] Moderation flows now require `forum_topics:moderate` or
    `forum_replies:moderate` at the service layer, not only in transport.
  - [x] Regression tests cover customer-denied mutation/moderation paths.
- [x] Validate observability, runbooks, and operational readiness for the final
  forum-owned persistence model.
- [x] Move forum tags to module-owned relation storage backed by shared taxonomy.
  - [x] forum-owned `forum_topic_tags` relation table is in place.
  - [x] topic create/update flows now resolve existing global terms before
    creating forum-local ones.
  - [x] external topic `tags: Vec<String>` contract remains unchanged.
- [x] Close forum slug/locale contract for the split track.
  - [x] category slug is translation-local and follows the resolved locale.
  - [x] topic slug remains a stable thread label copied into new translations for
    compatibility.
  - [x] current public forum API remains ID-based; any future slug lookup must
    reuse the shared locale fallback contract.

## Tracking and updates

When updating `rustok-forum` architecture, API contracts, tenancy behavior,
routing, or observability expectations:

1. Update this file for forum-local behavior and backlog.
2. Update `crates/rustok-forum/README.md` and `crates/rustok-forum/docs/README.md`
   when public behavior changes.
3. Update `docs/index.md` links if documentation structure changes.
4. If module responsibilities change, update `docs/modules/registry.md`
   accordingly.

## Checklist

- [x] Contract tests cover the current public use-cases.
