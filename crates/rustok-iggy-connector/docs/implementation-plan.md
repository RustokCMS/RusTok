# rustok-iggy-connector module implementation plan (`rustok-iggy-connector`)

## Scope and objective

This document captures the current implementation plan for `rustok-iggy-connector` in RusToK and
serves as the source of truth for rollout sequencing in `crates/rustok-iggy-connector`.

Primary objective: evolve `rustok-iggy-connector` in small, testable increments while preserving
compatibility with platform-level contracts.

## Target architecture

- `rustok-iggy-connector` remains focused on its bounded context and public crate API.
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
- [x] Align error/validation conventions with platform guidance.
- [x] Expand automated tests around core invariants and boundary behavior.
- [x] Implement `IggyConnector` trait with `connect`, `publish`, `subscribe`, `shutdown` methods.
- [x] Implement `RemoteConnector` for external Iggy server connection.
- [x] Implement `EmbeddedConnector` for embedded Iggy server.
- [x] Add optional Iggy SDK support via feature flags.

### Phase 2 — Domain expansion (in progress)

- [ ] Implement full Iggy SDK integration in RemoteConnector.
- [ ] Implement full embedded server lifecycle management.
- [ ] Add consumer group support for scalable consumption.
- [ ] Add message batching for high-throughput scenarios.
- [ ] Standardize cross-module integration points and events.
- [ ] Document ownership and release gates for new capabilities.

### Phase 3 — Productionization (planned)

- [ ] Finalize rollout and migration strategy for incremental adoption.
- [ ] Complete security/tenancy/rbac checks relevant to the module.
- [ ] Validate observability, runbooks, and operational readiness.
- [ ] Add health checks and metrics.

## Current Status

The connector provides:
- Trait-based abstraction (`IggyConnector`)
- Two implementations: `RemoteConnector` and `EmbeddedConnector`
- Config structs for both modes
- Partition calculation for message routing
- Message subscriber abstractions
- Comprehensive error handling

Optional Iggy SDK integration via `iggy` feature flag for full functionality.

## Tracking and updates

When updating `rustok-iggy-connector` architecture, API contracts, tenancy behavior, routing,
or observability expectations:

1. Update this file first.
2. Update `crates/rustok-iggy-connector/README.md` and `crates/rustok-iggy-connector/docs/README.md` when public behavior changes.
3. Update `docs/index.md` links if documentation structure changes.
4. If module responsibilities change, update `docs/modules/registry.md` accordingly.
