# rustok-content module implementation plan (`rustok-content`)

## Scope and objective

This document captures the current implementation plan for `rustok-content` in RusToK and
serves as the source of truth for rollout sequencing in `crates/rustok-content`.

Primary objective: evolve `rustok-content` in small, testable increments while preserving
compatibility with platform-level contracts.

## Target architecture

- `rustok-content` remains focused on its bounded context and public crate API.
- Integrations with other modules go through stable interfaces in `rustok-core`
  (or dedicated integration crates where applicable).
- Behavior changes are introduced through additive, backward-compatible steps.
- Observability and operability requirements are part of delivery readiness.

## Delivery phases

### Phase 0 — Foundation (done)

- [x] Baseline crate/module structure is in place.
- [x] Base docs and registry presence are established.
- [x] Core compile-time integration with the workspace is available.

**Exit criteria**
- [x] API contract frozen.
- [x] Sanitizer coverage for foundational input boundaries.
- [x] RBAC matrix documented for foundational operations.
- [x] Event/reindex integration wired for baseline domain events.
- [x] Migration rollback plan captured for bootstrap schema setup.

### Phase 1 — Contract hardening (done)

- [x] Freeze public API expectations for the current module surface.
- [x] Align error/validation conventions with platform guidance.
- [x] Expand automated tests around core invariants and boundary behavior.

**Exit criteria**
- [x] API contract frozen. (`CRATE_API.md` with all required sections; verified by `contract_tests.rs`)
- [x] Sanitizer coverage is enforced for orchestration command payloads. (`ensure_safe_text` / `ensure_safe_optional_text` / `ensure_idempotency_key` called on all inputs in `ContentOrchestrationService`)
- [x] RBAC matrix is complete for moderation/create cross-domain actions. (`ensure_scope` enforces `Action::Moderate` + `Action::Create` on all four orchestration methods)
- [x] Event/reindex integration is covered by minimal integration/e2e tests. (`tests/node_event_index_integration_test.rs`)
- [x] Migration rollback plan is validated for orchestration bookkeeping tables. (`down()` in `m20260311_000001_create_content_orchestration_tables.rs` drops both tables)

### Phase 2 — Domain expansion (planned)

- [ ] Implement prioritized domain capabilities for `rustok-content`.
- [ ] Standardize cross-module integration points and events.
- [ ] Document ownership and release gates for new capabilities.

**Exit criteria**
- [ ] API contract frozen.
- [ ] Sanitizer coverage includes newly introduced domain payloads.
- [ ] RBAC matrix reflects all new resource/action combinations.
- [ ] Event/reindex integration includes runbook-backed failure handling.
- [ ] Migration rollback plan exists for all newly introduced tables/indexes.

### Phase 3 — Productionization (planned)

- [ ] Finalize rollout and migration strategy for incremental adoption.
- [ ] Complete security/tenancy/rbac checks relevant to the module.
- [ ] Validate observability, runbooks, and operational readiness.

**Exit criteria**
- [ ] API contract frozen and versioned with explicit deprecation policy.
- [ ] Sanitizer coverage is measured and included in release gates.
- [ ] RBAC matrix is validated against runtime enforcement tests.
- [ ] Event/reindex integration is proven in production-like drills.
- [ ] Migration rollback plan is rehearsed and documented in runbooks.

## Tracking and updates

When updating `rustok-content` architecture, API contracts, tenancy behavior, routing,
or observability expectations:

1. Update this file first.
2. Update `crates/rustok-content/README.md` and `crates/rustok-content/docs/README.md` when public behavior changes.
3. Update `docs/index.md` links if documentation structure changes.
4. If module responsibilities change, update `docs/modules/registry.md` accordingly.

## Checklist

- [x] контрактные тесты покрывают все публичные use-case.

