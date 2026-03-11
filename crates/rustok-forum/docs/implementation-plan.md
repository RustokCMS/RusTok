# rustok-forum module implementation plan (`rustok-forum`)

## Scope and objective

This document captures the current implementation plan for `rustok-forum` in RusToK and
serves as the source of truth for rollout sequencing in `crates/rustok-forum`.

Primary objective: evolve `rustok-forum` in small, testable increments while preserving
compatibility with platform-level contracts.

## Target architecture

- `rustok-forum` remains focused on its bounded context and public crate API.
- Integrations with other modules go through stable interfaces in `rustok-core`
  (or dedicated integration crates where applicable).
- Behavior changes are introduced through additive, backward-compatible steps.
- Observability and operability requirements are part of delivery readiness.

## Delivery phases

### Phase 0 — Foundation (done)

- [x] Baseline crate/module structure is in place.
- [x] Base docs and registry presence are established.
- [x] Core compile-time integration with the workspace is available.

### Phase 1 — Contract hardening (done)

- [x] Freeze public API expectations for the current module surface.
  - Public surface: `CategoryService`, `TopicService`, `ReplyService`, `ModerationService` with CRUD operations.
  - `ModerationService` extended with topic operations: `pin_topic`, `unpin_topic`, `lock_topic`, `unlock_topic`, `close_topic`, `archive_topic`.
- [x] Align error/validation conventions with platform guidance.
  - Empty title/body/content/name/slug in `create` methods return `ForumError::Validation`.
  - Error types follow platform `thiserror` conventions.
- [x] Expand automated tests around core invariants and boundary behavior.
  - 9 inline lib tests for `node_to_topic`, `node_to_category`, `node_to_reply` mapping logic.
  - 15 pure unit tests in `tests/unit.rs`: constants, error display, DTO serde defaults.
  - 2 module contract tests in `tests/module.rs`: metadata and migrations list.
  - Integration test scaffold in `tests/integration.rs` (ignored, requires DB).

### Phase 2 — Bug fixes and i18n hardening (done)

- [x] Fix P0: `author_id` is now propagated from `SecurityContext` to `CreateNodeInput` in `TopicService` and `ReplyService`.
- [x] Fix P0: `ReplyService::create` now validates topic status before creating a reply.
  - Returns `ForumError::TopicClosed` if topic status is `closed`.
  - Returns `ForumError::TopicArchived` if topic status is `archived`.
- [x] Fix P0: `TopicService::list` and `CategoryService::list` now fetch full node data to populate metadata fields.
  - `is_pinned`, `is_locked`, `forum_status`, `reply_count`, `category_id`, `slug` are now correctly populated from node metadata.
  - `icon`, `color`, `topic_count`, `reply_count` in `CategoryService::list` are correctly populated.
- [x] Fix P1: `locale.rs` module with `resolve_translation` / `resolve_body` / `available_locales` helpers.
  - Fallback chain: `requested → "en" → first available`.
  - All `node_to_*` mappers migrated to use the new helpers.
- [x] Fix P1: `effective_locale` and `available_locales` added to `TopicResponse`, `TopicListItem`, `CategoryResponse`, `CategoryListItem`.
- [x] Fix P1: `effective_locale` added to `ReplyResponse` and `ReplyListItem`.
- [x] Fix P1: `author_id: Option<Uuid>` added to `TopicResponse`, `TopicListItem`, `ReplyResponse`, `ReplyListItem`.
- [x] Fix P1: `slug: Option<String>` added to `CreateTopicInput`; `slug` included in node translation.
- [x] Fix P1: `ListRepliesFilter` (new DTO) replaces hard-coded `per_page: 200` in `list_for_topic`. Signature changed to accept filter.
- [x] Fix P1: Forum-specific `DomainEvent` variants added to `rustok-core`:
  - `ForumTopicCreated`, `ForumTopicReplied`, `ForumTopicStatusChanged`, `ForumTopicPinned`, `ForumReplyStatusChanged`.
  - All variants include `event_type()`, `schema_version()`, `validate()`, and `affects_index()` support.
- [x] Fix P1: `TopicService::create` publishes `ForumTopicCreated` after node creation.
- [x] Fix P1: `ReplyService::create` publishes `ForumTopicReplied` after node creation.
- [x] Fix P1: `ModerationService` publishes `ForumTopicStatusChanged`, `ForumTopicPinned`, `ForumReplyStatusChanged`.
- [x] Fix P1: `ModerationService` methods for `approve_reply`, `reject_reply`, `hide_reply`, `pin_topic`, `unpin_topic`, `close_topic`, `archive_topic` now accept `tenant_id: Uuid`.

### Phase 3 — Productionization (planned)

- [ ] Own database migrations for forum-specific tables:
  - `forum_category_stats` — denormalized category counters (topic_count, reply_count, last_post_at).
  - `forum_topic_votes` / `forum_reply_votes` — voting tables.
  - `forum_solutions` — Q&A solution marking.
  - `forum_subscriptions` — per-category/topic notification subscriptions.
  - `forum_user_stats` — per-user forum statistics.
  - `forum_moderation_log` — moderation audit log.
  - `forum_read_tracking` — read state per user/topic.
  - `forum_tags` / `forum_tag_translations` / `forum_topic_tags` — localized tags.
- [ ] Move `is_pinned`, `is_locked`, `forum_status`, `reply_count` out of `metadata` JSONB into typed columns.
- [ ] Atomic `reply_count` increment on reply creation (via UPDATE counter on topic node).
- [ ] RBAC checks in `ModerationService` (verify moderator/admin scope before mutation).
- [ ] `forum_read_tracking` integration — unread count per user per topic.
- [ ] Full security/tenancy/rbac checks relevant to the module.
- [ ] Validate observability, runbooks, and operational readiness.

## Tracking and updates

When updating `rustok-forum` architecture, API contracts, tenancy behavior, routing,
or observability expectations:

1. Update this file first.
2. Update `crates/rustok-forum/README.md` and `crates/rustok-forum/docs/README.md` when public behavior changes.
3. Update `docs/index.md` links if documentation structure changes.
4. If module responsibilities change, update `docs/modules/registry.md` accordingly.

## Checklist

- [x] контрактные тесты покрывают все публичные use-case.

