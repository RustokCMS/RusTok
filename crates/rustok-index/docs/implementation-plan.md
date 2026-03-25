# rustok-index module implementation plan (`rustok-index`)

## Scope and objective

This document captures the current implementation plan for `rustok-index` in RusToK and
serves as the source of truth for rollout sequencing in `crates/rustok-index`.

Primary objective: evolve `rustok-index` into the platform's canonical cross-module
index/read-model capability while preserving compatibility with platform-level contracts.

## Canonical direction

- `rustok-index` is a `Core` infrastructure module.
- Its primary role is not end-user search, but denormalized indexing, ingestion,
  link-aware querying, and cross-module read support.
- The target model is closer to Medusa's `Index Module`: a central indexed data store
  that enables efficient querying/filtering across otherwise isolated module boundaries.
- Search concerns should live in a separate search capability/module. The current crate
  still contains early search-oriented scaffolding, and that should be treated as
  transitional technical debt to extract over time.
- `rustok-index` must still be brought to a production-usable, fully working state as
  a real index module, not left as a placeholder.

## Target architecture

- `rustok-index` remains focused on bounded, infrastructure-level indexing concerns.
- It owns canonical ingestion contracts, index rebuild/sync workflows, and cross-module
  read/index storage needed for relational filtering.
- Integrations with domain modules go through stable interfaces in `rustok-core`
  (or dedicated integration crates where applicable).
- Search consumers may depend on `rustok-index` projections, but `rustok-index` itself
  should not become the product-facing search module.
- Observability and operability requirements are part of delivery readiness.

## Delivery phases

### Phase 0 — Foundation (done)

- [x] Baseline crate/module structure is in place.
- [x] Base docs and registry presence are established.
- [x] Core compile-time integration with the workspace is available.

### Phase 1 — Contract hardening (planned)

- [ ] Freeze the canonical role of `rustok-index` as the platform index/read-model module.
- [ ] Align error/validation conventions with platform guidance.
- [ ] Expand automated tests around ingestion, rebuild, sync, and boundary behavior.

### Phase 2 — Domain expansion (planned)

- [ ] Implement prioritized index capabilities for cross-module denormalized reads.
- [ ] Standardize ingestion and synchronization contracts across modules.
- [ ] Document ownership and release gates for new indexing capabilities.

### Phase 3 — Working index module rollout (planned)

- [ ] Make `rustok-index` production-usable for real cross-module query scenarios.
- [ ] Expose stable query/index read surfaces for platform consumers.
- [ ] Add operator tooling, health checks, and rebuild controls.

### Phase 4 — Search decoupling (planned)

- [ ] Identify search-oriented code currently living in `rustok-index`.
- [ ] Extract search-specific runtime/API concerns into a dedicated search module/capability.
- [ ] Keep `rustok-index` as the canonical source for indexed relations/read-model data
  where the future search module benefits from it.

### Phase 5 — Productionization (planned)

- [ ] Finalize rollout and migration strategy for incremental adoption.
- [ ] Complete security/tenancy/rbac checks relevant to the module.
- [ ] Validate observability, runbooks, and operational readiness.

## Dedicated index track

This section isolates the intended long-term role of `rustok-index`.

### Goal

Adopt a central index/read-model module that:

- ingests data from multiple platform modules into a denormalized store;
- supports efficient filtering across module boundaries;
- acts as a stable infrastructure substrate for cross-module queries;
- is actually usable by host applications and other modules in production;
- remains independent from product-facing full-text search concerns.

### Proposed implementation phases

#### Phase IX0 — Contract freeze

- [ ] Freeze the canonical purpose of `rustok-index`.
- [ ] Define which entities and links are ingested into the index store.
- [ ] Define ingestion lifecycle: bootstrap, incremental sync, rebuild, retry.
- [ ] Define how tenant and locale scoping are represented in indexed records.

#### Phase IX1 — Ingestion and synchronization

- [ ] Standardize event-driven ingestion from core and optional modules.
- [ ] Add idempotent sync/rebuild flows.
- [ ] Add failure handling, retries, and operational visibility for ingestion.
- [ ] Document how index freshness is measured and enforced.

#### Phase IX2 — Link-aware queries

- [ ] Define the canonical query API for cross-module filtering.
- [ ] Support querying by linked entities and denormalized attributes.
- [ ] Ensure pagination, filtering, and count semantics are stable.
- [ ] Add contract tests for linked-data query scenarios.

#### Phase IX2.5 — Working platform surfaces

- [ ] Expose a stable internal service/query surface that `apps/server` and future
  consumers can use without depending on storage details.
- [ ] Decide whether `rustok-index` also exposes GraphQL/REST/admin diagnostics surfaces,
  or remains internal-only with host-owned operator endpoints.
- [ ] Add practical query flows for the first real use cases:
  - filter entities by linked entities;
  - filter by denormalized attributes coming from multiple modules;
  - paginate and sort indexed results;
  - retrieve counts efficiently.
- [ ] Add migrations/schema needed for production-scale index storage and link tables.
- [ ] Add consistency verification tooling between source modules and indexed state.

#### Phase IX3 — Separation from search

- [ ] Inventory all search-related code paths currently present in this crate.
- [ ] Decide which parts stay as shared indexing primitives versus move to search.
- [ ] Extract product-facing search APIs and engines to a dedicated search module.
- [ ] Keep migration compatibility during the transition.

#### Phase IX4 — Operational readiness

- [ ] Add telemetry for sync lag, rebuild duration, failed ingestion, and consistency drift.
- [ ] Add operator tooling for rebuild and health verification.
- [ ] Document runbooks for stale index, partial rebuild, and sync failure scenarios.

## First production slice for `rustok-index`

Recommended first useful slice:

- ingest entities from the first selected modules;
- ingest selected cross-module links;
- persist denormalized indexed records in a central relational store;
- expose stable query/filter/count operations over indexed data;
- support bootstrap sync, incremental sync, rebuild, and health verification;
- provide operator visibility for lag/failures/rebuild status.

This gives the platform a genuinely working index module before any future search
module builds on top of it.

## Open questions

- Should `rustok-index` expose only internal services, or also a public query API for
  other modules and host applications?
- Which cross-module relations are mandatory for the first useful version?
- Should the future search module read only from `rustok-index`, or may it also accept
  direct domain-specific ingestion paths where justified?

## Risks

- Leaving search and index concerns mixed together will blur module boundaries and make
  future extraction more expensive.
- Over-coupling `rustok-index` to one product flow can weaken its value as shared
  infrastructure.
- Without a strict ingestion contract, cross-module query consistency may drift.

## Tracking and updates

When updating `rustok-index` architecture, API contracts, tenancy behavior, routing,
or observability expectations:

1. Update this file first.
2. Update `crates/rustok-index/README.md` and `crates/rustok-index/docs/README.md` when public behavior changes.
3. Update `docs/index.md` links if documentation structure changes.
4. If module responsibilities change, update `docs/modules/registry.md` accordingly.

## Checklist

- [x] контрактные тесты покрывают все публичные use-case.
