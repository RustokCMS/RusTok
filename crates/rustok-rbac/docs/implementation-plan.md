# rustok-rbac module implementation plan (`rustok-rbac`)

## Scope and objective

This document captures the current implementation plan for `rustok-rbac` in RusToK and
serves as the source of truth for rollout sequencing in `crates/rustok-rbac`.

Primary objective: evolve `rustok-rbac` in small, testable increments while preserving
compatibility with platform-level contracts.

## Target architecture

- `rustok-rbac` remains focused on its bounded context and public crate API.
- Integrations with other modules go through stable interfaces in `rustok-core`
  (or dedicated integration crates where applicable).
- Behavior changes are introduced through additive, backward-compatible steps.
- Observability and operability requirements are part of delivery readiness.

## Delivery phases

### Phase 0 — Foundation (done)

- [x] Baseline crate/module structure is in place.
- [x] Base docs and registry presence are established.
- [x] Core compile-time integration with the workspace is available.

### Phase 1 — Contract hardening (in progress)

- [ ] Freeze public API expectations for the current module surface.
- [x] Introduce shared permission-policy helpers (`permission_policy`) and start consuming them from `apps/server` extractors/service wiring to reduce server-owned policy logic.
- [ ] Align error/validation conventions with platform guidance.
- [ ] Expand automated tests around core invariants and boundary behavior.

### Phase 2 — Domain expansion (planned)

- [ ] Implement prioritized domain capabilities for `rustok-rbac`.
- [ ] Standardize cross-module integration points and events.
- [ ] Document ownership and release gates for new capabilities.

### Phase 3 — Productionization (planned)

- [ ] Finalize rollout and migration strategy for incremental adoption.
- [ ] Complete security/tenancy/rbac checks relevant to the module.
- [ ] Validate observability, runbooks, and operational readiness.

## Tracking and updates

When updating `rustok-rbac` architecture, API contracts, tenancy behavior, routing,
or observability expectations:

1. Update this file first.
2. Update `crates/rustok-rbac/README.md` and `crates/rustok-rbac/docs/README.md` when public behavior changes.
3. Update `docs/index.md` links if documentation structure changes.
4. If module responsibilities change, update `docs/modules/registry.md` accordingly.
